use std::{path::PathBuf, sync::Arc, time::Duration};

use bevy::app::{AppExit, ScheduleRunnerPlugin};
use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{save_to_disk, Capturing, Screenshot, ScreenshotCaptured};
use bevy::winit::WinitPlugin;
use kuluu_render::graphics_settings::GraphicsSettings;
use kuluu_render::hud::classic_status::{
    update_classic_status_system, ClassicFrame, ClassicGaugeRow,
};
use kuluu_render::hud::panel_column::layout_panel_column_system;
use kuluu_render::hud::party_frame::{
    spawn_party_frames, update_party_frame_system, PartyFrameSettings, PartyRowTarget,
};
use kuluu_render::ui_font::{apply_ui_font, load_ui_font, UiFont};
use kuluu_render::{nameplate_color::NameColorTable, scene::Target, snapshot::SceneState};
use kuluu_snapshot::{PartyMember, SceneSnapshot};

const PIXEL_CHANNELS: usize = 4;
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 1080;
const SETTLE_FRAMES: u32 = 60;
const CASE_FRAMES: u32 = 80;
const CASE_SCALES: &[f32] = &[1.0, 1.5, 2.0];
const WINDOW_STYLES: usize = kuluu_render::ui_element_atlas::WINDOW_STYLE_COUNT as usize;
const STEP_SECONDS: f64 = 1.0 / 60.0;
const TEST_ZONE: u16 = 1;

#[derive(Resource)]
struct CaptureTarget {
    image: Handle<Image>,
    directory: PathBuf,
}

fn main() {
    let root =
        ffxi_dat::DatRoot::from_env_or_default().expect("original UI resource install is required");
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    ..default()
                })
                .disable::<WinitPlugin>(),
        )
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            STEP_SECONDS,
        )))
        .add_plugins((
            kuluu_render::ui_element_atlas::UiElementAtlasPlugin,
            kuluu_render::hud::retail_frame::RetailFramePlugin,
            kuluu_render::hud::retail_text::RetailMenuLabelsPlugin,
        ))
        .insert_resource(kuluu_render::ui_element_atlas::UiElementDatRoot(Some(
            Arc::new(root),
        )))
        .insert_resource(kuluu_render::input_mode::InputMode::Menu(
            kuluu_render::input_mode::MenuStack::root(),
        ))
        .init_resource::<kuluu_render::hud::menu::DynamicMenu>()
        .init_resource::<kuluu_render::hud::HudPanels>()
        .init_resource::<kuluu_render::hud::network_status::NetStatusVisible>()
        .init_resource::<kuluu_render::audio::AudioMuteState>()
        .init_resource::<GraphicsSettings>()
        .init_resource::<PartyFrameSettings>()
        .init_resource::<NameColorTable>()
        .init_resource::<Target>()
        .init_resource::<SceneState>()
        .init_resource::<UiFont>()
        .add_systems(
            Startup,
            (
                setup,
                spawn_party_frames,
                load_ui_font,
                kuluu_render::hud::menu::spawn_main_menu,
            ),
        )
        .add_systems(
            Update,
            (
                drive,
                update_party_frame_system,
                update_classic_status_system,
                kuluu_render::hud::menu::update_main_menu,
                kuluu_render::hud::menu::sync_original_command_labels,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            layout_panel_column_system.after(bevy::ui::UiSystems::Layout),
        )
        .add_systems(
            PostUpdate,
            apply_ui_font.before(bevy::ui::UiSystems::Content),
        )
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT;
    let image = images.add(image);
    let directory = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("kuluu-original-ui-capture"));
    std::fs::create_dir_all(&directory).expect("capture directory must be writable");
    commands.spawn((
        Camera2d,
        IsDefaultUiCamera,
        RenderTarget::Image(image.clone().into()),
    ));
    commands.insert_resource(CaptureTarget { image, directory });
}

fn fixture(party: bool) -> SceneSnapshot {
    let mut snapshot = SceneSnapshot {
        self_char_id: Some(1),
        char_name: Some("Trivius".into()),
        zone_id: Some(TEST_ZONE),
        stage: kuluu_snapshot::Stage::InZone,
        ..default()
    };
    let names = if party {
        vec![
            "Trivius",
            "Longmembernamexx",
            "Caster",
            "Outofzone",
            "Lowhealth",
            "Tank",
        ]
    } else {
        vec!["Trivius"]
    };
    for (index, name) in names.into_iter().enumerate() {
        let id = index as u32 + 1;
        let zone_no = if name == "Outofzone" {
            TEST_ZONE + 1
        } else {
            TEST_ZONE
        };
        snapshot.party.push(PartyMember {
            id,
            act_index: index as u16,
            name: Some(name.into()),
            hp: if party { 1234 } else { 37 },
            mp: if party { 456 } else { 0 },
            tp: 1000,
            hp_pct: if name == "Lowhealth" { 5 } else { 100 },
            mp_pct: if party { 50 } else { 0 },
            zone_no,
            main_job: 0,
            main_job_lv: 0,
            sub_job: 0,
            sub_job_lv: 0,
            is_party_leader: index == 0 && party,
            is_alliance_leader: false,
            party_no: if party {
                0
            } else {
                ffxi_proto::decode::NO_PARTY
            },
            in_mog_house: false,
        });
        snapshot.entities.push(
            serde_json::from_value(serde_json::json!({
                "id": id, "act_index": index, "kind": "pc", "name": name,
                "pos": { "x": index, "y": 0, "z": 0 }, "heading": 0,
                "hp_pct": 100, "bt_target_id": 0, "char_flags": kuluu_snapshot::CharFlags { new_character: true, ..default() }
            }))
            .expect("fixture entity must deserialize"),
        );
    }
    snapshot
}

fn drive(
    mut commands: Commands,
    mut frame: Local<u32>,
    mut state: ResMut<SceneState>,
    mut scale: ResMut<UiScale>,
    target: Res<CaptureTarget>,
    text: Query<(Entity, &Text, &ComputedNode, &UiGlobalTransform)>,
    parents: Query<&ChildOf>,
    rows: Query<(), Or<(With<PartyRowTarget>, With<ClassicGaugeRow>)>>,
    capturing: Query<(), With<Capturing>>,
    mut exit: MessageWriter<AppExit>,
    mut window_style: ResMut<kuluu_render::hud::retail_frame::RetailWindowStyle>,
    menu_rows: Query<(
        &kuluu_render::hud::menu::MainMenuRow,
        &Node,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
    menus: Query<
        (&Node, &ComputedNode, &UiGlobalTransform),
        With<kuluu_render::hud::menu::MainMenu>,
    >,
    status_frames: Query<(&Node, &ComputedNode, &UiGlobalTransform), With<ClassicFrame>>,
) {
    let case = *frame / CASE_FRAMES;
    let step = *frame % CASE_FRAMES;
    *frame += 1;
    if case as usize >= CASE_SCALES.len() * WINDOW_STYLES {
        if capturing.is_empty() {
            exit.write(AppExit::Success);
        }
        return;
    }
    if step == 0 {
        scale.0 = CASE_SCALES[case as usize % CASE_SCALES.len()];
        window_style.0 = (case as usize / CASE_SCALES.len() + 1) as u8;
        state.snapshot = fixture(case as usize % 2 != 0);
        state.dirty = true;
    }
    if step != SETTLE_FRAMES {
        return;
    }
    let row_bounds = menu_rows
        .iter()
        .filter(|(_, node, _, _)| node.display != Display::None)
        .map(|(row, _, computed, transform)| {
            assert!(
                computed.size().min_element() > 0.0,
                "original menu rows must reserve their authored size"
            );
            (
                row.slot,
                Rect::from_center_size(transform.affine().translation, computed.size()),
            )
        })
        .collect::<Vec<_>>();
    assert!(
        !row_bounds.is_empty(),
        "production Command menu must render"
    );
    for (_, menu, menu_transform) in menus
        .iter()
        .filter(|(node, _, _)| node.display != Display::None)
    {
        let menu_rect = Rect::from_center_size(menu_transform.affine().translation, menu.size());
        for (_, status, status_transform) in status_frames
            .iter()
            .filter(|(node, _, _)| node.display != Display::None)
        {
            let status_rect =
                Rect::from_center_size(status_transform.affine().translation, status.size());
            let overlap = menu_rect.intersect(status_rect);
            assert!(
                overlap.width() <= 0.0 || overlap.height() <= 0.0,
                "Command menu covers the status display"
            );
        }
    }
    for (index, (slot, rect)) in row_bounds.iter().enumerate() {
        for (other_slot, other_rect) in &row_bounds[index + 1..] {
            let overlap = rect.intersect(*other_rect);
            assert!(
                overlap.width() <= 0.0 || overlap.height() <= 0.0,
                "original menu rows overlap: {slot} and {other_slot}"
            );
        }
    }
    let mut bounds = Vec::new();
    for (entity, value, node, transform) in &text {
        if value.is_empty() || node.size().min_element() <= 0.0 {
            continue;
        }
        let mut ancestor = entity;
        while let Ok(parent) = parents.get(ancestor) {
            ancestor = parent.parent();
            if rows.contains(ancestor) {
                let rect = Rect::from_center_size(transform.affine().translation, node.size());
                bounds.push((ancestor, value.0.clone(), rect));
                break;
            }
        }
    }
    assert!(
        !bounds.is_empty(),
        "production party text must have computed bounds"
    );
    assert!(
        !bounds.iter().any(|(_, value, _)| value == "New"),
        "classic entries must omit custom activity text"
    );
    for label in ["HP", "MP", "TP"] {
        assert!(
            bounds.iter().any(|(_, value, _)| value == label),
            "self gauge label must render: {label}"
        );
    }
    for (index, (row, label, rect)) in bounds.iter().enumerate() {
        for (other_row, other_label, other_rect) in &bounds[index + 1..] {
            if row == other_row {
                let overlap = rect.intersect(*other_rect);
                assert!(
                    overlap.width() <= 0.0 || overlap.height() <= 0.0,
                    "party text overlaps: {label} and {other_label}"
                );
            }
        }
    }
    let mode = if case as usize >= CASE_SCALES.len() {
        "party"
    } else {
        "solo"
    };
    let name = format!(
        "original-style-{}-{mode}-{}.png",
        window_style.0,
        (scale.0 * 100.0) as u32
    );
    commands
        .spawn(Screenshot::image(target.image.clone()))
        .observe(save_to_disk(target.directory.join(name)))
        .observe(|capture: On<ScreenshotCaptured>| {
            let pixels = capture
                .image
                .data
                .as_deref()
                .expect("capture pixels must exist");
            let first = pixels
                .first_chunk::<PIXEL_CHANNELS>()
                .expect("capture pixels must not be empty");
            assert!(
                pixels
                    .chunks_exact(PIXEL_CHANNELS)
                    .any(|pixel| pixel != first),
                "uniform capture cannot verify the HUD"
            );
        });
    println!(
        "PASS: party text does not overlap at {}% UI scale",
        scale.0 * 100.0
    );
}
