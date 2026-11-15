//! Astral Cobalt materials for the in-game windows. Input and scene data stay with each window.
use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

use super::chat_panel::{ChatRowBody, ChatRowSpan};
use super::menu::{MainMenuPane, MainMenuRow, MainMenuTitle};
use super::retail_frame::RetailWindow;
use super::style::theme;
use crate::input_mode::{InputMode, MenuKind};

#[derive(Resource)]
pub struct AstralCobalt;

#[derive(Component)]
pub struct EquipmentHeader;

#[derive(Asset, AsBindGroup, TypePath, Clone)]
struct AstralMaterial {
    // x selects the surface. y is focus progress and z is confirm progress.
    #[uniform(0)]
    effect: Vec4,
}

impl UiMaterial for AstralMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://kuluu_render/hud/astral.wesl".into()
    }
}

#[derive(Resource)]
struct Surfaces {
    heading_font: Handle<Font>,
    body_font: Handle<Font>,
    window: Handle<AstralMaterial>,
    row: Handle<AstralMaterial>,
    focus: Handle<AstralMaterial>,
    confirm: Handle<AstralMaterial>,
}

impl FromWorld for Surfaces {
    fn from_world(world: &mut World) -> Self {
        let mut fonts = world.resource_mut::<Assets<Font>>();
        let heading_font = fonts.add(Font::from_bytes(
            include_bytes!("../../assets/fonts/DejaVuSerifCondensed-BoldItalic.ttf").to_vec(),
        ));
        let body_font = fonts.add(Font::from_bytes(
            include_bytes!("../../assets/fonts/DejaVuSansCondensed.ttf").to_vec(),
        ));
        let mut assets = world.resource_mut::<Assets<AstralMaterial>>();
        let mut make = |kind| {
            assets.add(AstralMaterial {
                effect: Vec4::new(kind, 1., 1., 0.),
            })
        };
        Self {
            heading_font,
            body_font,
            window: make(0.),
            row: make(1.),
            focus: make(2.),
            confirm: make(0.),
        }
    }
}

fn skin_text(
    surfaces: Res<Surfaces>,
    mut labels: Query<
        (&mut TextFont, Has<MainMenuRow>, Has<MainMenuTitle>),
        (
            Added<TextFont>,
            Or<(
                With<MainMenuRow>,
                With<MainMenuTitle>,
                With<ChatRowBody>,
                With<ChatRowSpan>,
                With<super::classic_status::ClassicText>,
                With<super::equipment_screen::EquipText>,
            )>,
        ),
    >,
) {
    for (mut font, row, title) in &mut labels {
        let handle = if row || title {
            &surfaces.heading_font
        } else {
            &surfaces.body_font
        };
        font.font = bevy::text::FontSource::Handle(handle.clone());
        if row {
            font.font_size = 12.0.into();
        }
        if title {
            font.font_size = 11.0.into();
        }
    }
}

fn skin_windows(
    mut commands: Commands,
    surfaces: Res<Surfaces>,
    windows: Query<Entity, Added<RetailWindow>>,
) {
    for entity in &windows {
        commands.entity(entity).insert((
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            MaterialNode(surfaces.window.clone()),
        ));
    }
}

#[derive(Default)]
struct Motion {
    selection: Option<(MenuKind, usize)>,
    depth: usize,
    focus_start: f64,
    confirm_start: Option<f64>,
}

fn update_menu(
    mut commands: Commands,
    time: Res<Time>,
    mode: Option<Res<InputMode>>,
    surfaces: Res<Surfaces>,
    mut materials: ResMut<Assets<AstralMaterial>>,
    mut motion: Local<Motion>,
    rows: Query<(Entity, &TextColor, Option<&MaterialNode<AstralMaterial>>), With<MainMenuRow>>,
    headers: Query<
        (Entity, Has<EquipmentHeader>, &MaterialNode<AstralMaterial>),
        Or<(With<MainMenuPane>, With<EquipmentHeader>)>,
    >,
) {
    let Some(InputMode::Menu(stack)) = mode.as_deref() else {
        motion.selection = None;
        motion.depth = 0;
        motion.confirm_start = None;
        return;
    };
    let Some(level) = stack.current() else { return };
    let now = time.elapsed_secs_f64();
    let selection = Some((level.kind, level.cursor));
    if motion.selection != selection {
        motion.focus_start = now;
        motion.selection = selection;
    }
    let depth = stack.levels.len();
    if depth > motion.depth && motion.depth > 0 {
        motion.confirm_start = Some(now);
    } else if depth < motion.depth {
        motion.confirm_start = None;
    }
    motion.depth = depth;
    let progress = |start: f64, duration: f64| ((now - start) / duration).clamp(0., 1.) as f32;
    let focus = progress(motion.focus_start, 0.33);
    let confirm = motion
        .confirm_start
        .map_or(1., |start| progress(start, 0.84));
    for (handle, effect) in [
        (&surfaces.focus, Vec4::new(2., focus, 1., 0.)),
        (&surfaces.confirm, Vec4::new(0., 1., confirm, 0.)),
    ] {
        if materials
            .get(handle)
            .is_some_and(|mat| mat.effect != effect)
        {
            materials.get_mut(handle).unwrap().effect = effect;
        }
    }
    for (entity, color, material) in &rows {
        let handle = if color.0 == theme::CURSOR {
            &surfaces.focus
        } else {
            &surfaces.row
        };
        if material.is_none_or(|mat| mat.0 != *handle) {
            commands.entity(entity).insert(MaterialNode(handle.clone()));
        }
    }
    for (entity, equipment, material) in &headers {
        let is_current =
            equipment == matches!(level.kind, MenuKind::Equipment | MenuKind::EquipSlot(_));
        let handle = if confirm < 1. && is_current {
            &surfaces.confirm
        } else {
            &surfaces.window
        };
        if material.0 != *handle {
            commands.entity(entity).insert(MaterialNode(handle.clone()));
        }
    }
}

pub struct AstralCobaltPlugin;

impl Plugin for AstralCobaltPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "astral.wesl");
        app.insert_resource(AstralCobalt)
            .add_plugins(UiMaterialPlugin::<AstralMaterial>::default())
            .init_resource::<Surfaces>()
            .add_systems(
                PostUpdate,
                (skin_windows, skin_text, update_menu)
                    .chain()
                    .before(bevy::ui::UiSystems::Prepare),
            );
    }
}
