//! Cache of retail menu UI-element sprites, keyed by (group-name, index).
//! Backed by ffxi_dat::ui_element; mirrors the item/status icon caches
//! (hud/item_dat_root.rs, hud/status_ribbon.rs).

use std::collections::HashMap;
use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use ffxi_dat::texture::{DecodedTexture, TexFormat};
use ffxi_dat::ui_element::{
    crop_sprite, find_texture, find_ui_element_group, ui_sprite, UiSprite, UI_SHEET_FILE_ID,
};
use ffxi_dat::DatRoot;

// The four "static resource" menu UI DATs. XIM hardcodes their ROM paths
// (research/xim/src/jsMain/kotlin/xim/poc/UiResourceManager.kt UiResourceManager uiDats — ROM/0/13, ROM/119/51,
// ROM/280/15, ROM/324/95); these are those paths reverse-mapped through
// VTABLE/FTABLE to file ids, the version-stable handle, so an install whose
// patch level shuffles the physical ROM layout still resolves. The
// day-of-week orbs and weather element icons live in id 39542 (ROM/119/51).
pub const UI_DAT_FILE_IDS: [u32; 4] = [13, UI_SHEET_FILE_ID, 39551, 39560];
pub const WINDOW_STYLE_COUNT: u8 = 8;
const BASE_MENU_FILE_ID: u32 = 1;

pub fn read_ui_dats(root: &DatRoot) -> Vec<(u32, Vec<u8>)> {
    UI_DAT_FILE_IDS
        .into_iter()
        .filter_map(|id| {
            let loc = root.resolve(id).ok()?;
            let bytes = std::fs::read(loc.path_under(root)).ok()?;
            Some((id, bytes))
        })
        .collect()
}

const FRAMES_JP: &str = "menu    frames  ";
const FRAMES_US: &str = "menu    framesus";

// research/XIClient/src/XIClient/source/UI/UIManager.cpp UIManager::InitDraw.
const UI_ALPHA_MODULATE_2X: f32 = 2.0;

// Bound allocations for malformed DAT component sizes.
const MAX_TILED_UI_PIXELS: usize = 1_048_576;

#[derive(Resource, Default, Clone)]
pub struct UiElementDatRoot(pub Option<Arc<DatRoot>>);

#[derive(Resource, Default)]
pub struct UiElementAtlas {
    source_root: Option<Arc<DatRoot>>,
    dats: Vec<Arc<Vec<u8>>>,
    style_dat: Option<Arc<Vec<u8>>>,
    window_style: u8,
    generation: u64,
    loaded: bool,
    unavailable: bool,
    layouts: HashMap<&'static str, Option<Arc<ffxi_dat::menu_layout::MenuLayout>>>,
    sprites: HashMap<(String, usize), Option<Handle<Image>>>,
    elements: HashMap<(String, usize), Option<Vec<UiElementQuad>>>,
}

#[derive(Clone)]
pub struct UiElementQuad {
    pub image: Handle<Image>,
    pub rect: Rect,
    pub color: Color,
}

impl UiElementAtlas {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn set_window_style(&mut self, style: u8) {
        let selected = style.clamp(1, WINDOW_STYLE_COUNT);
        if self.window_style.clamp(1, WINDOW_STYLE_COUNT) != selected {
            self.style_dat = None;
            self.loaded = false;
            self.unavailable = false;
            self.sprites.clear();
            self.elements.clear();
            self.generation = self.generation.wrapping_add(1);
        }
        self.window_style = selected;
    }

    pub fn ensure_layout(
        &mut self,
        name: &'static str,
        root: &UiElementDatRoot,
    ) -> Option<Arc<ffxi_dat::menu_layout::MenuLayout>> {
        self.sync_root(root);
        if let Some(layout) = self.layouts.get(name) {
            return layout.clone();
        }
        let layout = self
            .ensure_dats(root)
            .iter()
            .find_map(|bytes| ffxi_dat::menu_layout::find_menu_layout(bytes, name).map(Arc::new));
        self.layouts.insert(name, layout.clone());
        layout
    }

    pub fn ensure_element(
        &mut self,
        group: &str,
        index: usize,
        dat_root: &UiElementDatRoot,
        images: &mut Assets<Image>,
    ) -> Option<Vec<UiElementQuad>> {
        self.sync_root(dat_root);
        let key = (group.to_string(), index);
        if let Some(slot) = self.elements.get(&key) {
            return slot.clone();
        }
        self.ensure_dats(dat_root);
        let quads = self.dats.iter().find_map(|bytes| {
            let resource = find_ui_element_group(bytes, group)?;
            resource
                .elements
                .get(index)?
                .components
                .iter()
                .map(|component| {
                    let texture = find_texture(bytes, &component.texture_ref)
                        .or_else(|| {
                            self.style_dat
                                .as_ref()
                                .and_then(|style| find_texture(style, &component.texture_ref))
                        })
                        .or_else(|| {
                            self.dats.iter().find_map(|candidate| {
                                find_texture(candidate, &component.texture_ref)
                            })
                        })?;
                    let mut sprite = crop_ui_sprite(
                        &texture,
                        component.uv_offset_x,
                        component.uv_offset_y,
                        component.uv_width,
                        component.uv_height,
                        component.flip_mode,
                    )?;
                    let mut color = crate::nameplate_color::quad_color(component.colors[0]);
                    if component.colors.iter().any(|c| *c != component.colors[0]) {
                        modulate_vertex_colors(&mut sprite, component.colors, texture.format_tag);
                        color = Color::WHITE;
                    } else if texture.format_tag == TexFormat::Dxt3 {
                        modulate_dxt3_ui_alpha(&mut sprite, color.alpha());
                        color.set_alpha(1.0);
                    }
                    let points = component
                        .positions
                        .map(|(x, y)| Vec2::new(f32::from(x), f32::from(y)));
                    Some(UiElementQuad {
                        image: upload_sprite(sprite, images),
                        rect: Rect::from_corners(
                            points
                                .into_iter()
                                .fold(Vec2::splat(f32::INFINITY), Vec2::min),
                            points
                                .into_iter()
                                .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max),
                        ),
                        color,
                    })
                })
                .collect::<Option<Vec<_>>>()
        });
        self.elements.insert(key, quads.clone());
        quads
    }

    pub fn ensure(
        &mut self,
        group: &str,
        index: usize,
        dat_root: &UiElementDatRoot,
        images: &mut Assets<Image>,
    ) -> Option<Handle<Image>> {
        self.sync_root(dat_root);
        let key = (group.to_string(), index);
        if let Some(slot) = self.sprites.get(&key) {
            return slot.clone();
        }
        let handle = self
            .ensure_dats(dat_root)
            .iter()
            .find_map(|bytes| resolve_sprite(bytes, group, index))
            .map(|sprite| upload_sprite(sprite, images));
        self.sprites.insert(key, handle.clone());
        handle
    }

    fn sync_root(&mut self, dat_root: &UiElementDatRoot) {
        let unchanged = match (&self.source_root, &dat_root.0) {
            (Some(cached), Some(current)) => Arc::ptr_eq(cached, current),
            (None, None) => true,
            _ => false,
        };
        if !unchanged {
            *self = Self {
                source_root: dat_root.0.clone(),
                window_style: self.window_style,
                generation: self.generation.wrapping_add(1),
                ..default()
            };
        }
    }

    fn ensure_dats(&mut self, dat_root: &UiElementDatRoot) -> &[Arc<Vec<u8>>] {
        if self.loaded || self.unavailable {
            return &self.dats;
        }
        let Some(root) = dat_root.0.as_ref() else {
            self.unavailable = true;
            return &self.dats;
        };
        self.dats = read_ui_dats(root)
            .into_iter()
            .map(|(_, bytes)| Arc::new(bytes))
            .collect();
        if let Ok(location) = root.resolve(BASE_MENU_FILE_ID) {
            if let Ok(bytes) = std::fs::read(location.path_under(root)) {
                self.dats.push(Arc::new(bytes));
            }
        }
        let style_id =
            UI_DAT_FILE_IDS[0] + u32::from(self.window_style.clamp(1, WINDOW_STYLE_COUNT));
        self.style_dat = root
            .resolve(style_id)
            .ok()
            .and_then(|location| std::fs::read(location.path_under(root)).ok().map(Arc::new));
        self.loaded = true;
        self.unavailable = self.dats.is_empty();
        &self.dats
    }
}

fn crop_ui_sprite(
    texture: &DecodedTexture,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    flip_mode: u8,
) -> Option<UiSprite> {
    let pixel_count = usize::from(width).checked_mul(usize::from(height))?;
    if pixel_count == 0
        || pixel_count > MAX_TILED_UI_PIXELS
        || texture.width == 0
        || texture.height == 0
    {
        return None;
    }
    if let Some(sprite) = crop_sprite(texture, x, y, width, height, flip_mode) {
        return Some(sprite);
    }
    let mut rgba = Vec::with_capacity(pixel_count.checked_mul(4)?);
    for row in 0..u32::from(height) {
        for col in 0..u32::from(width) {
            let source_x = (u32::from(x) + col) % texture.width;
            let source_y = (u32::from(y) + row) % texture.height;
            let source = ((source_y * texture.width + source_x) * 4) as usize;
            rgba.extend_from_slice(texture.rgba.get(source..source + 4)?);
        }
    }
    let tiled = DecodedTexture {
        width: u32::from(width),
        height: u32::from(height),
        format_tag: texture.format_tag,
        rgba,
    };
    crop_sprite(&tiled, 0, 0, width, height, flip_mode)
}

fn modulate_vertex_colors(sprite: &mut UiSprite, raw: [[u8; 4]; 4], format: TexFormat) {
    let colors = raw.map(|color| {
        Vec4::from_array(
            crate::nameplate_color::quad_color(color)
                .to_linear()
                .to_f32_array(),
        )
    });
    let alpha_scale = if format == TexFormat::Dxt3 {
        UI_ALPHA_MODULATE_2X
    } else {
        1.0
    };
    for (index, pixel) in sprite.rgba.chunks_exact_mut(4).enumerate() {
        let x = ((index as u32 % sprite.width) as f32 + 0.5) / sprite.width as f32;
        let y = ((index as u32 / sprite.width) as f32 + 0.5) / sprite.height as f32;
        let tint = if x + y <= 1.0 {
            colors[0] * (1.0 - x - y) + colors[1] * x + colors[2] * y
        } else {
            colors[3] * (x + y - 1.0) + colors[1] * (1.0 - y) + colors[2] * (1.0 - x)
        };
        let source = Color::srgba_u8(pixel[0], pixel[1], pixel[2], pixel[3]).to_linear();
        let output = Color::linear_rgba(
            source.red * tint.x,
            source.green * tint.y,
            source.blue * tint.z,
            (source.alpha * tint.w * alpha_scale).min(1.0),
        );
        pixel.copy_from_slice(&output.to_srgba().to_u8_array());
    }
}

/// DXT3 decoding preserves raw alpha; palette decoding already doubles it.
/// Clamp after modulation so partially transparent vertices retain bright texels.
fn modulate_dxt3_ui_alpha(sprite: &mut UiSprite, vertex_alpha: f32) {
    for pixel in sprite.rgba.chunks_exact_mut(4) {
        pixel[3] = (f32::from(pixel[3]) * vertex_alpha * UI_ALPHA_MODULATE_2X)
            .round()
            .min(f32::from(u8::MAX)) as u8;
    }
}

// The US sheet names the group "menu    framesus" (measured on KNOWN_CLIENTS
// horizonxi-2023 and retail-2026-09); research/xim UiResourceManager.kt
// register aliases the JP "menu    frames  ".
fn resolve_sprite(bytes: &[u8], group: &str, index: usize) -> Option<UiSprite> {
    ui_sprite(bytes, group, index).or_else(|| {
        if group == FRAMES_JP {
            ui_sprite(bytes, FRAMES_US, index)
        } else {
            None
        }
    })
}

fn upload_sprite(sprite: UiSprite, images: &mut Assets<Image>) -> Handle<Image> {
    let mut image = Image::new(
        Extent3d {
            width: sprite.width,
            height: sprite.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        sprite.rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    images.add(image)
}

pub struct UiElementAtlasPlugin;

impl Plugin for UiElementAtlasPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiElementDatRoot>()
            .init_resource::<UiElementAtlas>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_alpha_gradient_remains_visible_when_the_first_vertex_is_transparent() {
        let colors = [
            [127, 127, 127, 0],
            [127, 127, 127, 127],
            [127, 127, 127, 0],
            [127, 127, 127, 127],
        ];
        let source = UiSprite {
            width: 2,
            height: 1,
            rgba: vec![255; 8],
        };
        let mut palette = source.clone();
        modulate_vertex_colors(&mut palette, colors, TexFormat::Argb32);
        assert_eq!(palette.rgba, [255, 255, 255, 48, 255, 255, 255, 144]);
        let mut dxt3 = source;
        modulate_vertex_colors(&mut dxt3, colors, TexFormat::Dxt3);
        assert_eq!(dxt3.rgba, [255, 255, 255, 96, 255, 255, 255, 255]);
    }

    #[test]
    fn menu_components_wrap_uv_coordinates_instead_of_stretching_the_texture() {
        let texture = DecodedTexture {
            width: 2,
            height: 2,
            format_tag: TexFormat::Argb32,
            rgba: vec![10, 0, 0, 255, 20, 0, 0, 255, 30, 0, 0, 255, 40, 0, 0, 255],
        };
        assert!(crop_sprite(&texture, 1, 1, 4, 3, 0).is_none());
        let wrapped = crop_ui_sprite(&texture, 1, 1, 4, 3, 0).unwrap();
        assert_eq!((wrapped.width, wrapped.height), (4, 3));
        assert_eq!(
            wrapped
                .rgba
                .chunks_exact(4)
                .map(|pixel| pixel[0])
                .collect::<Vec<_>>(),
            [40, 30, 40, 30, 20, 10, 20, 10, 40, 30, 40, 30]
        );
        assert!(crop_ui_sprite(&texture, 0, 0, u16::MAX, u16::MAX, 0).is_none());
    }

    #[test]
    fn dxt3_ui_alpha_modulates_before_saturating() {
        let mut sprite = UiSprite {
            width: 4,
            height: 1,
            rgba: vec![
                20, 40, 60, 0, 20, 40, 60, 85, 20, 40, 60, 136, 20, 40, 60, 255,
            ],
        };
        let vertex_alpha = crate::nameplate_color::quad_color([127; 4]).alpha();
        modulate_dxt3_ui_alpha(&mut sprite, vertex_alpha);
        assert_eq!(
            sprite.rgba,
            [20, 40, 60, 0, 20, 40, 60, 128, 20, 40, 60, 205, 20, 40, 60, 255],
            "transparent texels stay clear, RGB is untouched, and alpha saturates after modulation"
        );
    }

    fn test_dat_root() -> Option<UiElementDatRoot> {
        let root = ffxi_dat::archive::open_test_install()?;
        Some(UiElementDatRoot(Some(Arc::new(root))))
    }

    // Gated on a retail install (self-skips). Exercises the whole viewer-side
    // path against real data: load the UI DATs, resolve the frames->framesus
    // alias, decode + crop, and upload a 14x14 day-orb image into Assets.
    #[test]
    fn real_dat_day_orb_uploads_14x14() {
        let Some(dat_root) = test_dat_root() else {
            return;
        };
        let mut images = Assets::<Image>::default();
        let mut atlas = UiElementAtlas::default();

        let handle = atlas
            .ensure(FRAMES_JP, 106, &dat_root, &mut images)
            .expect("Firesday orb resolves via the frames->framesus alias");
        let image = images.get(&handle).expect("uploaded image present");
        assert_eq!(image.width(), 14);
        assert_eq!(image.height(), 14);

        // Second lookup is served from the cache (same handle).
        let again = atlas.ensure(FRAMES_JP, 106, &dat_root, &mut images);
        assert_eq!(again.as_ref(), Some(&handle));
    }

    // Gated on a retail install (self-skips). The weather-icon widget resolves
    // "font    usgaiji " 0-7 (the eight element icons, ROM/119/51.DAT); pin
    // that every index uploads as a 16x16 sprite.
    #[test]
    fn real_dat_usgaiji_weather_icons_upload_16x16() {
        let Some(dat_root) = test_dat_root() else {
            return;
        };
        let mut images = Assets::<Image>::default();
        let mut atlas = UiElementAtlas::default();

        for index in 0..8 {
            let handle = atlas
                .ensure("font    usgaiji ", index, &dat_root, &mut images)
                .unwrap_or_else(|| panic!("usgaiji element {index} resolves"));
            let image = images.get(&handle).expect("uploaded image present");
            assert_eq!((image.width(), image.height()), (16, 16), "index {index}");
        }
    }

    #[test]
    fn real_dat_command_title_loads_wrapped_trim_and_original_letters() {
        let Some(dat_root) = test_dat_root() else {
            return;
        };
        let mut images = Assets::<Image>::default();
        let mut atlas = UiElementAtlas::default();
        assert!(atlas
            .ensure_element(
                "menu    comwin",
                16,
                &UiElementDatRoot::default(),
                &mut images
            )
            .is_none());
        let mut trim_images = Vec::new();
        for style in 1..=WINDOW_STYLE_COUNT {
            atlas.set_window_style(style);
            let quads = atlas
                .ensure_element("menu    comwin", 16, &dat_root, &mut images)
                .expect("original Commands title resolves after install selection");
            assert_eq!(quads.len(), 11);
            assert!(quads.iter().all(|quad| images.contains(&quad.image)));
            assert_eq!(quads[0].rect, Rect::new(27.0, 0.0, 84.0, 4.0));
            assert_eq!(quads[3].rect, Rect::new(26.0, -3.0, 34.0, 6.0));
            let trim = images.get(&quads[0].image).unwrap();
            assert_eq!(trim.width(), 57);
            trim_images.push(trim.data.clone());
            let generation = atlas.generation();
            atlas.set_window_style(style);
            assert_eq!(atlas.generation(), generation);
            let cached = atlas
                .ensure_element("menu    comwin", 16, &dat_root, &mut images)
                .unwrap();
            assert_eq!(cached[0].image, quads[0].image);
        }
        assert!(trim_images.windows(2).any(|pair| pair[0] != pair[1]));
        assert!(atlas
            .ensure_element(
                "menu    comwin",
                16,
                &UiElementDatRoot::default(),
                &mut images
            )
            .is_none());
    }
}
