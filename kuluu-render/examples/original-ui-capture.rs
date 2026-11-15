use std::{path::PathBuf, sync::Arc, time::Duration};

use bevy::app::{AppExit, ScheduleRunnerPlugin};
use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{save_to_disk, Capturing, Screenshot, ScreenshotCaptured};
use bevy::winit::WinitPlugin;
use kuluu_render::graphics_settings::GraphicsSettings;
use kuluu_render::hud::classic_status::{
    update_classic_status_system, ClassicFrame, ClassicGaugeRow,
};
use kuluu_render::hud::menu::{MainMenuPane, MainMenuRow, MainMenuTitle};
use kuluu_render::hud::network_status;
use kuluu_render::hud::panel_column::layout_panel_column_system;
use kuluu_render::hud::party_frame::{
    spawn_party_frames, update_party_frame_system, PartyFrameSettings, PartyRowTarget,
};
use kuluu_render::hud::retail_text::{RetailMenuCursor, RetailMenuLabel, WINDOW_LABEL_GROUP};
use kuluu_render::hud::{chat_panel, item_dat_root};
use kuluu_render::input_mode::{InputMode, MenuKind, MenuStack};
use kuluu_render::ui_font::{apply_ui_font, load_ui_font, UiFont};
use kuluu_render::{nameplate_color::NameColorTable, scene::Target, snapshot::SceneState};
use kuluu_snapshot::{CharStats, ChatChannel, ChatLine, NetStats, PartyMember, SceneSnapshot};

const PIXEL_CHANNELS: usize = 4;
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 1080;
const SETTLE_FRAMES: u32 = 60;
const CASE_FRAMES: u32 = 80;
const CASE_SCALES: &[f32] = &[1.0, 1.5, 2.0];
const WINDOW_STYLES: usize = kuluu_render::ui_element_atlas::WINDOW_STYLE_COUNT as usize;
// Observed in the owner's ROM/119/51.DAT. Keep this oracle independent of the renderer.
const ROOT_LABELS: [[usize; 12]; 2] = [
    [13, 15, 17, 19, 680, 21, 20, 28, 18, 16, 25, 23],
    [24, 26, 22, 31, 30, 29, 27, 36, 34, 766, 35, 33],
];
const CASE_COUNT: usize = WINDOW_STYLES * CASE_SCALES.len() * ROOT_LABELS.len();
const STEP_SECONDS: f64 = 1.0 / 60.0;
const TEST_ZONE: u16 = 1;

#[derive(Resource)]
struct CaptureTarget {
    image: Handle<Image>,
    directory: PathBuf,
}

fn main() {
    let mode = std::env::args().nth(2);
    let root = match mode.as_deref() {
        Some("cobalt-fallback") => None,
        None | Some("cobalt") | Some("classic-hud") => Some(Arc::new(
            ffxi_dat::DatRoot::from_env_or_default()
                .expect("original UI resource install is required"),
        )),
        _ => panic!("usage: original-ui-capture [directory] [classic-hud|cobalt|cobalt-fallback]"),
    };
    let mut app = App::new();
    app.add_plugins(
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
    .insert_resource(kuluu_render::ui_element_atlas::UiElementDatRoot(
        root.clone(),
    ))
    .insert_resource(InputMode::Menu(MenuStack::root()))
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
            network_status::spawn_network_status,
            load_ui_font,
            kuluu_render::hud::menu::spawn_main_menu,
        ),
    )
    .add_systems(
        Update,
        (
            update_party_frame_system,
            update_classic_status_system,
            network_status::update_network_status,
            network_status::apply_net_status_visibility,
            kuluu_render::hud::menu::update_main_menu,
            kuluu_render::hud::menu::sync_original_command_labels,
        )
            .chain(),
    )
    .add_systems(
        PostUpdate,
        network_status::sync_network_layout
            .before(kuluu_render::hud::retail_frame::OriginalFrameUpdate),
    )
    .add_systems(
        PostUpdate,
        layout_panel_column_system.after(bevy::ui::UiSystems::Layout),
    )
    .add_systems(
        PostUpdate,
        apply_ui_font.before(bevy::ui::UiSystems::Content),
    );
    if mode.is_some() {
        use kuluu_render::hud;
        if mode.as_deref() != Some("classic-hud") {
            app.add_plugins(hud::astral::AstralCobaltPlugin);
        }
        app.insert_resource(item_dat_root::ItemDatRoot(root))
            .init_resource::<item_dat_root::ItemIconCache>()
            .init_resource::<chat_panel::ChatScroll>()
            .init_resource::<chat_panel::BattleScroll>()
            .init_resource::<chat_panel::DebugScroll>()
            .init_resource::<chat_panel::ActiveChatTab>()
            .init_resource::<chat_panel::ChatUnread>()
            .init_resource::<chat_panel::ChatAutoSwitch>()
            .init_resource::<hud::item_screen::ItemScreenContainer>()
            .init_resource::<hud::item_detail::ItemMenuFocus>()
            .init_resource::<hud::check_view::CheckTarget>()
            .init_resource::<hud::auction::AuctionScreenState>()
            .init_resource::<hud::auction::AuctionSellInventory>()
            .init_resource::<hud::shop::ShopScreenState>()
            .add_systems(
                Startup,
                (
                    hud::spawn_bottom_left_stack,
                    hud::menu_help_bar::spawn_menu_help_bar,
                    setup_viewport,
                ),
            )
            .add_systems(PreUpdate, drive_cobalt)
            .add_systems(
                Update,
                (
                    chat_panel::apply_chat_layout,
                    chat_panel::update_chat_panel,
                    chat_panel::update_chat_tab_visuals_system,
                    hud::menu_help_bar::update_menu_help_bar,
                )
                    .chain(),
            );
        app.add_systems(PostStartup, hide_fixture_world_tools);
        hud::equipment_screen::install_equipment_screen_capture(&mut app);
    } else {
        app.add_systems(Update, drive.before(update_party_frame_system));
    }
    app.run();
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
        net_stats: NetStats {
            send_bps: 309,
            recv_bps: 196,
            send_health: 100,
            recv_health: 100,
        },
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

// The production chat layout reads the primary window's logical dimensions. The
// headless camera still renders to the image target, independently of this window.
fn setup_viewport(mut commands: Commands) {
    commands.spawn((
        Window {
            resolution: (WIDTH, HEIGHT).into(),
            ..default()
        },
        bevy::window::PrimaryWindow,
    ));
}

// This fixture has no world camera, map, or server clock. Omit the adjacent
// world tools so their uninitialized minimap and compass do not cover the HUD.
fn hide_fixture_world_tools(mut tools: Query<&mut Node, With<kuluu_render::hud::ChatTools>>) {
    for mut node in &mut tools {
        node.display = Display::None;
    }
}

#[derive(SystemParam)]
struct CommandHeading<'w, 's> {
    title: Query<'w, 's, (&'static ComputedNode, &'static UiGlobalTransform), With<MainMenuTitle>>,
    rows: Query<
        'w,
        's,
        (
            &'static MainMenuRow,
            &'static ComputedNode,
            &'static UiGlobalTransform,
        ),
    >,
}

impl CommandHeading<'_, '_> {
    fn assert_above_first_row(&self) {
        let (title, title_transform) = self.title.single().expect("visible Commands title");
        let (_, row, row_transform) = self
            .rows
            .iter()
            .find(|(r, _, _)| r.slot == 0)
            .expect("visible Status row");
        let title_bottom = title_transform.translation.y + title.size().y * 0.5;
        let row_top = row_transform.translation.y - row.size().y * 0.5;
        assert!(
            title_bottom <= row_top + 1.0,
            "Commands title overlaps first row: {title_bottom} > {row_top}"
        );
    }
}

const COBALT_CASES: &[&str] = &["solo-chat", "party-chat", "equipment", "equipment-updated"];

fn drive_cobalt(
    mut commands: Commands,
    mut frame: Local<u32>,
    mut state: ResMut<SceneState>,
    mut mode: ResMut<InputMode>,
    target: Res<CaptureTarget>,
    text: Query<(&Text, &ComputedNode)>,
    chat: Query<&TextSpan, With<chat_panel::ChatRowSpan>>,
    astral: Option<Res<kuluu_render::hud::astral::AstralCobalt>>,
    heading: CommandHeading,
    capturing: Query<(), With<Capturing>>,
    mut exit: MessageWriter<AppExit>,
) {
    let case = (*frame / CASE_FRAMES) as usize;
    let step = *frame % CASE_FRAMES;
    *frame += 1;
    let Some(name) = COBALT_CASES.get(case) else {
        if capturing.is_empty() {
            exit.write(AppExit::Success);
        }
        return;
    };
    if step == 0 {
        let mut snapshot = fixture(case != 0);
        let me = &mut snapshot.party[0];
        me.hp = 837;
        me.mp = 126;
        me.tp = 1140;
        me.hp_pct = 83;
        me.mp_pct = 70;
        me.main_job = 5;
        me.main_job_lv = 50;
        snapshot.stats = Some(CharStats {
            str_: 43,
            dex: 41,
            vit: 39,
            agi: 36,
            int_: 47,
            mnd: 48,
            chr: 38,
            hp_max: 1008,
            mp_max: 180,
            attack: 161,
            defense: 104,
            ..default()
        });
        snapshot.equipped[0] = Some(16535); // Bronze Sword, main hand.
        snapshot.chat = [
            (ChatChannel::Say, "Trivius", "Ready when you are."),
            (ChatChannel::Party, "Caster", "Meet at the gate."),
            (
                ChatChannel::Battle,
                "",
                "Trivius hits the Huge Hornet for 23 points of damage.",
            ),
        ]
        .into_iter()
        .map(|(channel, sender, text)| ChatLine {
            channel,
            sender: sender.into(),
            text: text.into(),
            ..default()
        })
        .collect();
        state.snapshot = snapshot;
        state.dirty = true;
        let mut stack = MenuStack::root();
        stack.current_mut().unwrap().cursor = 1;
        if case >= 2 {
            stack.push(MenuKind::Equipment);
        }
        *mode = if case == 0 {
            InputMode::World
        } else {
            InputMode::Menu(stack)
        };
    }
    // Exercise a second snapshot with the window already open. A stale or
    // fixture-painted item title must not survive the equipped-slot change.
    if case == 3 && step == 1 {
        state.snapshot.equipped[0] = Some(16536); // Iron Sword, main hand.
        state.dirty = true;
    }
    if step == SETTLE_FRAMES {
        assert!(
            chat.iter().any(|span| span.0.contains("Meet at the gate.")),
            "missing party chat"
        );
        let rendered = |needle: &str| {
            text.iter()
                .any(|(t, node)| t.0.contains(needle) && node.size().min_element() > 0.0)
        };
        assert!(
            rendered("837") && rendered("126") && rendered("1140"),
            "missing self vitals"
        );
        if case == 1 && astral.is_some() {
            heading.assert_above_first_row();
        }
        if case != 0 {
            assert!(rendered("Longmembernamexx"), "missing party roster");
        }
        if case >= 2 {
            let item = if case == 2 {
                "Bronze Sword"
            } else {
                "Iron Sword"
            };
            assert!(rendered(item), "missing equipped main-hand item {item}");
            assert!(rendered("837/1008"), "missing Equipment vitals");
        }
        commands
            .spawn(Screenshot::image(target.image.clone()))
            .observe(save_to_disk(target.directory.join(format!("{name}.png"))));
        println!("Validated HUD fixture state: {name}");
    }
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
    mut mode: ResMut<InputMode>,
    menu: CommandArt,
    status_frames: Query<(&Node, &ComputedNode, &UiGlobalTransform), With<ClassicFrame>>,
) {
    let case = (*frame / CASE_FRAMES) as usize;
    let step = *frame % CASE_FRAMES;
    *frame += 1;
    if case >= CASE_COUNT {
        if capturing.is_empty() {
            exit.write(AppExit::Success);
        }
        return;
    }
    let page = case % ROOT_LABELS.len();
    let scale_case = case / ROOT_LABELS.len();
    let party = !scale_case.is_multiple_of(2);
    let selected_slot = if party {
        ROOT_LABELS[page].len() - 1
    } else {
        0
    };
    if step == 0 {
        scale.0 = CASE_SCALES[scale_case % CASE_SCALES.len()];
        window_style.0 = (scale_case / CASE_SCALES.len() + 1) as u8;
        let InputMode::Menu(stack) = &mut *mode else {
            panic!("capture requires the root Commands menu");
        };
        stack.current_mut().expect("root menu level").cursor =
            page * ROOT_LABELS[0].len() + selected_slot;
        state.snapshot = fixture(party);
        state.dirty = true;
    }
    if step != SETTLE_FRAMES {
        return;
    }
    let (pane, row_bounds) = menu.validate(page, selected_slot, scale.0);
    let nominal = Rect::new(
        WIDTH as f32 - 128.0 * scale.0,
        48.0 * scale.0,
        WIDTH as f32 - 16.0 * scale.0,
        250.0 * scale.0,
    );
    let mut obstructed = false;
    for (node, status, transform) in &status_frames {
        if node.display == Display::None {
            continue;
        }
        let status = Rect::from_center_size(transform.affine().translation, status.size());
        let nominal_overlap = nominal.intersect(status);
        obstructed |= nominal_overlap.width() > 0.0 && nominal_overlap.height() > 0.0;
        let overlap = pane.intersect(status);
        assert!(
            overlap.width() <= 0.0 || overlap.height() <= 0.0,
            "Commands pane covers the status display"
        );
    }
    if obstructed {
        assert!(
            pane.min.x < nominal.min.x,
            "Commands pane must clear the tall roster"
        );
    } else {
        assert_pixel_position(pane.min, nominal.min, "unobstructed Commands position");
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
    let mode = if party { "party" } else { "solo" };
    let name = format!(
        "original-style-{}-page-{}-{mode}-{}.png",
        window_style.0,
        page + 1,
        (scale.0 * 100.0) as u32
    );
    commands
        .spawn(Screenshot::image(target.image.clone()))
        .observe(save_to_disk(target.directory.join(name)))
        .observe(move |capture: On<ScreenshotCaptured>| {
            let pixels = capture
                .image
                .data
                .as_deref()
                .expect("capture pixels must exist");
            assert_eq!(pixels.len(), (WIDTH * HEIGHT) as usize * PIXEL_CHANNELS);
            for rect in &row_bounds {
                assert!(
                    has_row_contrast(pixels, *rect),
                    "no visible command artwork in {rect:?}"
                );
            }
            let inactive_slot = usize::from(selected_slot == 0);
            assert!(
                plate_warmth(pixels, row_bounds[selected_slot]) > 0,
                "selected button needs a gold background"
            );
            assert!(
                plate_warmth(pixels, row_bounds[inactive_slot]) < 0,
                "inactive button needs a blue background"
            );
        });
    println!(
        "Validated Commands page {} at {}% UI scale in {mode} mode",
        page + 1,
        scale.0 * 100.0
    );
}

#[derive(SystemParam)]
struct CommandArt<'w, 's> {
    rows: Query<
        'w,
        's,
        (
            &'static MainMenuRow,
            &'static Node,
            &'static ComputedNode,
            &'static UiGlobalTransform,
            Option<&'static RetailMenuLabel>,
            Option<&'static RetailMenuCursor>,
            &'static TextColor,
        ),
    >,
    panes: Query<'w, 's, (&'static ComputedNode, &'static UiGlobalTransform), With<MainMenuPane>>,
    images: Query<'w, 's, (&'static ComputedNode, &'static UiGlobalTransform), With<ImageNode>>,
    network_art: Query<
        'w,
        's,
        (
            &'static network_status::NetworkSprite,
            &'static Node,
            &'static ComputedNode,
            &'static UiGlobalTransform,
        ),
    >,
    network: Query<
        'w,
        's,
        (&'static ComputedNode, &'static UiGlobalTransform),
        With<network_status::NetworkStatusPanel>,
    >,
}

impl CommandArt<'_, '_> {
    fn validate(&self, page: usize, selected_slot: usize, scale: f32) -> (Rect, Vec<Rect>) {
        let (pane, transform) = self.panes.single().expect("one Commands pane");
        let pane = Rect::from_center_size(transform.affine().translation, pane.size());
        assert_pixel_position(
            pane.size(),
            Vec2::new(112.0, 202.0) * scale,
            "Commands pane size",
        );

        let (network, transform) = self.network.single().expect("one network indicator");
        let network = Rect::from_center_size(transform.affine().translation, network.size());
        assert_pixel_position(
            Vec2::new(network.min.y, network.height()),
            Vec2::new(16.0, 30.0) * scale,
            "authored network height and top",
        );
        assert_pixel_position(
            Vec2::new(network.min.x, network.max.x),
            Vec2::new(pane.min.x, pane.max.x),
            "network and Commands horizontal alignment",
        );
        assert!(
            network.max.y <= pane.min.y,
            "network covers the Commands title"
        );

        let mut glyphs_per_line = [0; 3];
        for (sprite, node, computed, transform) in &self.network_art {
            let network_status::NetworkSprite::Character { line, .. } = *sprite else {
                continue;
            };
            if node.display == Display::None {
                continue;
            }
            let rect = Rect::from_center_size(transform.affine().translation, computed.size());
            let start = (rect.min - network.min) / scale;
            let end = (rect.max - network.min) / scale;
            // The title ends at y=6. S and R occupy separate rows on its right side.
            let inside = match line {
                0 => end.x <= 58.0 && start.y >= 6.0,
                1 => start.x >= 58.0 && start.y >= 6.0 && end.y <= 18.0,
                2 => start.x >= 58.0 && start.y >= 18.0,
                _ => false,
            };
            assert!(
                inside && end.y <= 30.0 && end.x <= 112.0,
                "network glyph overlaps or leaves its row: {start:?}..{end:?}"
            );
            glyphs_per_line[line] += 1;
        }
        assert_eq!(glyphs_per_line, [4, 4, 4], "expected 100%, S 309 and R 196");

        let mut row_bounds = Vec::new();
        for (row, node, computed, transform, label, cursor, color) in &self.rows {
            if node.display == Display::None {
                continue;
            }
            let slot = row.slot;
            let label = label.expect("every visible root row needs authored label art");
            assert_eq!(label.group, WINDOW_LABEL_GROUP);
            assert_eq!(
                label.index,
                ROOT_LABELS[page][slot],
                "wrong sprite on page {} row {}",
                page + 1,
                slot + 1
            );
            assert_eq!(
                color.0,
                Color::NONE,
                "fallback text is still visible at slot {slot}"
            );
            assert_eq!(
                cursor.is_some_and(|cursor| cursor.visible),
                slot == selected_slot,
                "wrong cursor at slot {slot}"
            );
            let rect = Rect::from_center_size(transform.affine().translation, computed.size());
            let offset = Vec2::new(16.0, 6.0 + slot as f32 * 16.0);
            assert_pixel_position(rect.min, pane.min + offset * scale, "command row origin");
            assert_pixel_position(
                rect.size(),
                Vec2::new(88.0, 16.0) * scale,
                "command row size",
            );
            for other in &row_bounds {
                let overlap = rect.intersect(*other);
                assert!(
                    overlap.width() <= 0.0 || overlap.height() <= 0.0,
                    "command row overlaps slot {slot}: {rect:?}, other {other:?}, intersection {overlap:?}"
                );
            }
            row_bounds.push(rect);
        }
        assert_eq!(
            row_bounds.len(),
            ROOT_LABELS[page].len(),
            "all twelve commands must render"
        );
        for (name, x, y, w, h) in [
            ("pane background", 0.0, 0.0, 112.0, 202.0),
            ("left page arrow", -16.0, 7.0, 15.0, 9.0),
            ("right page arrow", 113.0, 7.0, 15.0, 9.0),
            ("title line", 27.0, 0.0, 57.0, 4.0),
            (
                "selection cursor",
                -16.0,
                4.0 + 16.0 * selected_slot as f32,
                32.0,
                20.0,
            ),
        ] {
            let origin = pane.min + Vec2::new(x, y) * scale;
            let size = Vec2::new(w, h) * scale;
            let found = self.images.iter().any(|(node, transform)| {
                let rect = Rect::from_center_size(transform.affine().translation, node.size());
                (rect.min - origin).abs().max_element() <= 1.0
                    && (rect.size() - size).abs().max_element() <= 1.0
            });
            assert!(
                found,
                "missing artwork for {name} at {origin:?}, size {size:?}"
            );
        }
        row_bounds.sort_unstable_by(|a, b| a.min.y.total_cmp(&b.min.y));
        (pane, row_bounds)
    }
}

fn assert_pixel_position(actual: Vec2, expected: Vec2, label: &str) {
    // Each tested row and pane measurement lands on a whole physical pixel.
    assert!(
        (actual - expected).abs().max_element() < 0.01,
        "{label}: expected {expected:?}, found {actual:?}"
    );
}

fn has_row_contrast(pixels: &[u8], rect: Rect) -> bool {
    let mut darkest = u8::MAX;
    let mut brightest = u8::MIN;
    for y in rect.min.y.ceil() as usize..rect.max.y.floor() as usize {
        for x in rect.min.x.ceil() as usize..rect.max.x.floor() as usize {
            let start = (y * WIDTH as usize + x) * PIXEL_CHANNELS;
            let rgb = &pixels[start..start + 3];
            let value = *rgb.iter().max().expect("three color channels");
            darkest = darkest.min(value);
            brightest = brightest.max(value);
        }
    }
    brightest.saturating_sub(darkest) > 24
}

fn plate_warmth(pixels: &[u8], rect: Rect) -> i64 {
    // This patch of the observed buttons lies to the right of the selected labels.
    let start = rect.min + rect.size() * Vec2::new(0.90, 0.22);
    let end = rect.min + rect.size() * Vec2::new(0.94, 0.45);
    let mut red_minus_blue = 0_i64;
    for y in start.y.ceil() as usize..end.y.floor() as usize {
        for x in start.x.ceil() as usize..end.x.floor() as usize {
            let pixel = (y * WIDTH as usize + x) * PIXEL_CHANNELS;
            red_minus_blue += i64::from(pixels[pixel]) - i64::from(pixels[pixel + 2]);
        }
    }
    red_minus_blue
}
