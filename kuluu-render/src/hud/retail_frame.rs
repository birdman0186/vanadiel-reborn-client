use std::path::PathBuf;

use bevy::asset::{embedded_asset, RenderAssetUsages};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendState, Extent3d, RenderPipelineDescriptor, ShaderType, TextureDimension,
    TextureFormat,
};
use bevy::shader::ShaderRef;
use bevy::ui_render::ui_material::UiMaterialKey;
use ffxi_dat::texture::TexFormat;
use ffxi_dat::ui_element::{find_texture, find_ui_element_group, BlendFactor, UiElementComponent};

use crate::ui_element_atlas::{
    UiElementAtlas, UiElementDatRoot, UI_DAT_FILE_IDS, WINDOW_STYLE_COUNT,
};

const WINDOW_GROUP: &str = "menu    win00";
const PART_COUNT: usize = 10;
const CORNER_PART: usize = 1;
const HORIZONTAL_PART: usize = 2;
const VERTICAL_PART: usize = 4;
const DXT3_ALPHA_SCALE: f32 = 2.0;

#[derive(Component, Default)]
pub struct RetailWindow;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OriginalFrameUpdate;

#[derive(Resource, Clone, PartialEq, Eq)]
pub struct RetailWindowStyle(pub u8);

impl Default for RetailWindowStyle {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Clone, Debug, ShaderType)]
struct FramePart {
    uv: Vec4,
    native_size: Vec4,
    colors: [Vec4; 4],
    blend: UVec4,
}

#[derive(Clone, Debug, ShaderType)]
struct FrameUniform {
    metrics: Vec4,
    logical_size: Vec4,
    parts: [FramePart; PART_COUNT],
}

#[derive(Asset, AsBindGroup, Clone, Debug, TypePath)]
pub struct RetailFrameMaterial {
    #[uniform(0)]
    data: FrameUniform,
    #[texture(1)]
    #[sampler(2)]
    background: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    corner: Handle<Image>,
    #[texture(5)]
    #[sampler(6)]
    horizontal: Handle<Image>,
    #[texture(7)]
    #[sampler(8)]
    vertical: Handle<Image>,
}

impl UiMaterial for RetailFrameMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://kuluu_render/hud/retail_frame.wesl".into()
    }

    fn specialize(descriptor: &mut RenderPipelineDescriptor, _: UiMaterialKey<Self>) {
        if let Some(fragment) = descriptor.fragment.as_mut() {
            for target in fragment.targets.iter_mut().flatten() {
                target.blend = Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING);
            }
        }
    }
}

#[derive(Resource, Default)]
struct FrameCache {
    source: Option<(PathBuf, u8)>,
    material: Option<Handle<RetailFrameMaterial>>,
}

#[derive(Component)]
struct FrameSurface;

#[derive(Component)]
struct FrameFallback {
    background: BackgroundColor,
    border: BorderColor,
}

fn component_size(component: &UiElementComponent) -> Vec2 {
    let min = component
        .positions
        .iter()
        .map(|&(x, y)| Vec2::new(f32::from(x), f32::from(y)))
        .fold(Vec2::splat(f32::INFINITY), Vec2::min);
    let max = component
        .positions
        .iter()
        .map(|&(x, y)| Vec2::new(f32::from(x), f32::from(y)))
        .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
    max - min
}

fn load_frame(bytes: &[u8], images: &mut Assets<Image>) -> Option<RetailFrameMaterial> {
    let group = find_ui_element_group(bytes, WINDOW_GROUP)?;
    if group.elements.len() != PART_COUNT {
        return None;
    }
    let components = group
        .elements
        .iter()
        .map(|element| match element.components.as_slice() {
            [component] => Some(component),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    let mut parts = Vec::with_capacity(PART_COUNT);
    for component in &components {
        let size = component_size(component);
        if size.min_element() <= 0.0
            || component.flip_mode != 0
            || component.source_blend_factor != BlendFactor::Texture
            || component.blend_operation != BlendFactor::Frame
            || !matches!(
                component.dest_blend_factor,
                BlendFactor::Frame | BlendFactor::Zero
            )
        {
            return None;
        }
        let texture = find_texture(bytes, &component.texture_ref)?;
        let alpha_scale = if texture.format_tag == TexFormat::Dxt3 {
            DXT3_ALPHA_SCALE
        } else {
            1.0
        };
        parts.push(FramePart {
            uv: Vec4::new(
                f32::from(component.uv_offset_x),
                f32::from(component.uv_offset_y),
                f32::from(component.uv_width),
                f32::from(component.uv_height),
            ),
            native_size: size.extend(alpha_scale).extend(0.0),
            colors: component.colors.map(|rgba| {
                Vec4::from_array(
                    crate::nameplate_color::quad_color(rgba)
                        .to_linear()
                        .to_f32_array(),
                )
            }),
            blend: UVec4::new(
                u32::from(component.alpha_blend_enabled),
                component.source_blend_factor as u32,
                component.dest_blend_factor as u32,
                component.blend_operation as u32,
            ),
        });
    }
    let mut upload = |part: usize| {
        let texture = find_texture(bytes, &components[part].texture_ref)?;
        let mut image = Image::new(
            Extent3d {
                width: texture.width,
                height: texture.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            texture.rgba,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..ImageSamplerDescriptor::linear()
        });
        Some(images.add(image))
    };
    Some(RetailFrameMaterial {
        data: FrameUniform {
            metrics: Vec4::new(
                component_size(components[CORNER_PART]).x,
                component_size(components[CORNER_PART]).y,
                component_size(components[HORIZONTAL_PART]).y,
                component_size(components[VERTICAL_PART]).x,
            ),
            logical_size: Vec4::ZERO,
            parts: parts.try_into().ok()?,
        },
        background: upload(0)?,
        corner: upload(CORNER_PART)?,
        horizontal: upload(HORIZONTAL_PART)?,
        vertical: upload(VERTICAL_PART)?,
    })
}

fn update_frames(
    mut commands: Commands,
    root: Res<UiElementDatRoot>,
    style: Res<RetailWindowStyle>,
    mut cache: ResMut<FrameCache>,
    mut atlas: ResMut<UiElementAtlas>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<RetailFrameMaterial>>,
    mut windows: Query<
        (
            Entity,
            &mut BackgroundColor,
            &mut BorderColor,
            Option<&Children>,
            Option<&FrameFallback>,
        ),
        With<RetailWindow>,
    >,
    surfaces: Query<(), With<FrameSurface>>,
) {
    atlas.set_window_style(style.0);
    let source = root.0.as_ref().map(|root| {
        (
            root.root().to_path_buf(),
            style.0.clamp(1, WINDOW_STYLE_COUNT),
        )
    });
    let source_changed = source != cache.source || root.is_changed();
    if source_changed {
        for (_, _, _, children, _) in &windows {
            if let Some(children) = children {
                for child in children {
                    if surfaces.contains(*child) {
                        commands.entity(*child).despawn();
                    }
                }
            }
        }
        cache.source = source.clone();
        cache.material = source.and_then(|(_, selected)| {
            let root = root.0.as_ref()?;
            let file_id = UI_DAT_FILE_IDS[0] + u32::from(selected);
            let location = root.resolve(file_id).ok()?;
            let bytes = std::fs::read(location.path_under(root)).ok()?;
            let material = load_frame(&bytes, &mut images)?;
            Some(materials.add(material))
        });
    }
    let Some(material) = cache.material.as_ref() else {
        for (entity, mut background, mut border, _, fallback) in &mut windows {
            if let Some(fallback) = fallback {
                *background = fallback.background;
                *border = fallback.border;
                commands.entity(entity).remove::<FrameFallback>();
            }
        }
        return;
    };
    for (entity, mut background, mut border, children, fallback) in &mut windows {
        if fallback.is_none() {
            commands.entity(entity).insert(FrameFallback {
                background: *background,
                border: *border,
            });
        }
        background.0 = Color::NONE;
        *border = BorderColor::all(Color::NONE);
        if !source_changed
            && children
                .is_some_and(|children| children.iter().any(|child| surfaces.contains(child)))
        {
            continue;
        }
        let surface_material = materials.get(material).unwrap().clone();
        let surface_material = materials.add(surface_material);
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                FrameSurface,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ZIndex(-1),
                Pickable::IGNORE,
                MaterialNode(surface_material),
            ));
        });
    }
}

fn update_surface_size(
    surfaces: Query<
        (&ComputedNode, &MaterialNode<RetailFrameMaterial>, &ChildOf),
        With<FrameSurface>,
    >,
    authored: Query<(), With<super::retail_layout::OriginalBackdrop>>,
    mut materials: ResMut<Assets<RetailFrameMaterial>>,
) {
    for (node, handle, parent) in &surfaces {
        if let Some(mut material) = materials.get_mut(handle) {
            material.data.logical_size = (node.size() * node.inverse_scale_factor())
                .extend(f32::from(authored.contains(parent.parent())))
                .extend(0.0);
        }
    }
}

pub struct RetailFramePlugin;

impl Plugin for RetailFramePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "retail_frame.wesl");
        app.add_plugins(UiMaterialPlugin::<RetailFrameMaterial>::default())
            .init_resource::<RetailWindowStyle>()
            .init_resource::<FrameCache>()
            .add_systems(
                PostUpdate,
                super::equipment_screen::position_equipment_cells
                    .before(bevy::ui::UiSystems::Prepare),
            )
            .add_systems(
                PostUpdate,
                update_frames
                    .run_if(not(resource_exists::<super::astral::AstralCobalt>))
                    .in_set(OriginalFrameUpdate)
                    .before(bevy::ui::UiSystems::Prepare),
            )
            .add_systems(
                PostUpdate,
                update_surface_size.after(bevy::ui::UiSystems::Layout),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_resources_preserve_the_frame_and_late_resources_can_be_removed() {
        let mut app = App::new();
        app.init_resource::<UiElementDatRoot>()
            .init_resource::<UiElementAtlas>()
            .init_resource::<RetailWindowStyle>()
            .init_resource::<FrameCache>()
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<RetailFrameMaterial>>()
            .add_systems(Update, update_frames);
        let background = BackgroundColor(Color::WHITE);
        let border = BorderColor::all(Color::BLACK);
        let entity = app
            .world_mut()
            .spawn((RetailWindow, Node::default(), background, border))
            .id();
        app.update();
        assert_eq!(
            *app.world().get::<BackgroundColor>(entity).unwrap(),
            background
        );
        let Some(root) = ffxi_dat::archive::open_test_install() else {
            return;
        };
        app.world_mut().resource_mut::<UiElementDatRoot>().0 = Some(std::sync::Arc::new(root));
        app.update();
        assert_eq!(
            app.world().get::<BackgroundColor>(entity).unwrap().0,
            Color::NONE
        );
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<FrameSurface>>()
                .iter(app.world())
                .count(),
            1
        );
        app.world_mut().resource_mut::<RetailWindowStyle>().0 = WINDOW_STYLE_COUNT;
        app.update();
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<FrameSurface>>()
                .iter(app.world())
                .count(),
            1
        );
        app.world_mut().resource_mut::<UiElementDatRoot>().0 = None;
        app.update();
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<FrameSurface>>()
                .iter(app.world())
                .count(),
            0
        );
        assert_eq!(
            *app.world().get::<BackgroundColor>(entity).unwrap(),
            background
        );
        assert_eq!(*app.world().get::<BorderColor>(entity).unwrap(), border);
    }

    #[test]
    fn installed_window_styles_use_authored_geometry_colors_and_wrap() {
        let Some(root) = ffxi_dat::archive::open_test_install() else {
            return;
        };
        for style in 0..WINDOW_STYLE_COUNT {
            let path = root
                .resolve(UI_DAT_FILE_IDS[0] + u32::from(style) + 1)
                .unwrap()
                .path_under(&root);
            let bytes = std::fs::read(path).expect("original window style must be readable");
            let group = find_ui_element_group(&bytes, WINDOW_GROUP).unwrap();
            let mut images = Assets::default();
            let material = load_frame(&bytes, &mut images).expect("original frame must load");
            assert_eq!(material.data.parts.len(), group.elements.len());
            for (index, part) in material.data.parts.iter().enumerate() {
                let component = &group.elements[index].components[0];
                assert_eq!(
                    part.native_size.truncate().truncate(),
                    component_size(component)
                );
                assert_eq!(part.blend.x != 0, component.alpha_blend_enabled);
                for (source, uploaded) in component.colors.iter().zip(part.colors) {
                    assert_eq!(
                        uploaded,
                        Vec4::from_array(
                            crate::nameplate_color::quad_color(*source)
                                .to_linear()
                                .to_f32_array()
                        )
                    );
                }
            }
            assert_ne!(material.data.parts[0].colors, material.data.parts[9].colors);
            if style == 0 {
                assert_ne!(
                    material.data.parts[CORNER_PART].colors[0],
                    material.data.parts[CORNER_PART].colors[1]
                );
            }
            let image = images.get(&material.horizontal).unwrap();
            let ImageSampler::Descriptor(sampler) = &image.sampler else {
                panic!("frame must tile")
            };
            assert_eq!(sampler.address_mode_u, ImageAddressMode::Repeat);
            assert!(material.data.parts[HORIZONTAL_PART].uv.z > image.width() as f32);
        }
    }
}
