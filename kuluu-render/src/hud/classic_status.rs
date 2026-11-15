use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use kuluu_snapshot::{PartyMember, SceneSnapshot, Stage};

use super::panel_column::ColumnPanel;
use super::party_frame::{
    party_content_key, party_windows, PartyContentKey, PartyFrameSettings, PartyRowTarget,
};
use super::style::{self, theme};
use crate::components::InGameEntity;
use crate::nameplate_color::{ncol, NameColorTable};
use crate::scene::Target;
use crate::snapshot::SceneState;

// .agents/skills/retail-observe/references/classic-status-layout-2026-11-15.md Supported layout and Implementation limits.
const FONT_PX: f32 = 14.0;
const PADDING_PX: f32 = 4.0;
const GAP_PX: f32 = 2.0;
const GAUGE_HEIGHT_PX: f32 = 7.0;
const BORDER_PX: f32 = 1.0;
const HP_COLOR: Color = Color::srgb(1.0, 0.55, 0.65);
const MP_COLOR: Color = Color::srgb(0.9, 0.85, 0.25);
const TP_COLOR: Color = Color::srgb(0.4, 0.8, 0.9);
const TRACK_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.7);
const SELECTED_COLOR: Color = Color::srgba(0.35, 0.55, 0.95, 0.3);
const MISSING_VALUE: &str = "--";
const TP_UNITS_PER_PERCENT: u32 = 10;

#[derive(Component)]
pub struct ClassicGaugeRow;

#[derive(Component)]
pub(super) struct ClassicText;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum ClassicFrame {
    SelfStatus,
    Party(u8),
}

#[derive(Component)]
pub struct ClassicRows(ClassicFrame);

pub(super) fn spawn_classic_frames(commands: &mut Commands) {
    for frame in [
        ClassicFrame::SelfStatus,
        ClassicFrame::Party(0),
        ClassicFrame::Party(1),
        ClassicFrame::Party(2),
    ] {
        commands
            .spawn((
                InGameEntity,
                frame,
                match frame {
                    ClassicFrame::SelfStatus => ColumnPanel::SELF_HUD,
                    ClassicFrame::Party(_) => ColumnPanel::ROSTER,
                },
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(style::PANEL_COLUMN_RIGHT_PX),
                    bottom: Val::Px(style::PANEL_COLUMN_BOTTOM_PX),
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                super::retail_frame::RetailWindow,
                BackgroundColor(theme::FRAME_BG),
                BorderColor::all(theme::FRAME_EDGE),
            ))
            .with_children(|root| {
                root.spawn((
                    ClassicRows(frame),
                    Node {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                ));
            });
    }
}

fn self_member(snapshot: &SceneSnapshot) -> Option<&PartyMember> {
    snapshot
        .self_char_id
        .and_then(|id| snapshot.party.iter().find(|member| member.id == id))
}

fn party_visible(members: &[&PartyMember]) -> bool {
    members
        .iter()
        .any(|member| member.party_no != ffxi_proto::decode::NO_PARTY)
}

pub fn update_classic_status_system(
    mut commands: Commands,
    state: Res<SceneState>,
    target: Res<Target>,
    settings: Res<PartyFrameSettings>,
    colors: Res<NameColorTable>,
    mut roots: Query<(&ClassicFrame, &mut Node)>,
    hosts: Query<(Entity, &ClassicRows, Option<&Children>)>,
    mut last_key: Local<Option<PartyContentKey>>,
) {
    if !settings.classic_layout {
        for (_, mut node) in &mut roots {
            node.display = Display::None;
        }
        return;
    }
    if last_key.is_some()
        && !state.dirty
        && !target.is_changed()
        && !settings.is_changed()
        && !colors.is_changed()
    {
        return;
    }
    let snap = &state.snapshot;
    let key = party_content_key(snap, colors.generation(), false);
    if Some(&key) == last_key.as_ref() && !target.is_changed() && !settings.is_changed() {
        return;
    }
    *last_key = Some(key);
    let windows = party_windows(&snap.party, snap.self_char_id);
    let own = self_member(snap);
    let sc = settings.scale;
    for (frame, mut node) in &mut roots {
        let show = snap.stage != Stage::Zoning
            && match frame {
                ClassicFrame::SelfStatus => true,
                ClassicFrame::Party(index) => party_visible(&windows[usize::from(*index)]),
            };
        node.display = if show { Display::Flex } else { Display::None };
        node.width = Val::Px(style::PANEL_WIDTH_PX * sc);
        node.padding = UiRect::all(Val::Px(PADDING_PX * sc));
        node.border = UiRect::all(Val::Px(BORDER_PX * sc));
    }
    for (entity, host, children) in &hosts {
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        commands
            .entity(entity)
            .with_children(|parent| match host.0 {
                ClassicFrame::SelfStatus => spawn_self(parent, own, sc),
                ClassicFrame::Party(index) => {
                    let title = match index {
                        0 => "Party",
                        1 => "Party B",
                        _ => "Party C",
                    };
                    parent.spawn((
                        Text::new(title),
                        style::text_font(FONT_PX * sc),
                        TextColor(theme::TEXT),
                        TextLayout::justify(Justify::Center),
                    ));
                    for member in &windows[usize::from(index)] {
                        spawn_party_member(parent, member, snap, own, target.id, &colors, sc);
                    }
                }
            });
    }
}

fn label(parent: &mut ChildSpawnerCommands, text: String, color: Color, sc: f32) {
    parent.spawn((
        ClassicText,
        Text::new(text),
        style::text_font(FONT_PX * sc),
        TextColor(color),
        TextLayout {
            linebreak: LineBreak::NoWrap,
            ..default()
        },
        Node {
            flex_shrink: 0.0,
            ..default()
        },
    ));
}

fn gauge(parent: &mut ChildSpawnerCommands, pct: u8, color: Color, sc: f32) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                height: Val::Px(GAUGE_HEIGHT_PX * sc),
                border: UiRect::all(Val::Px(BORDER_PX * sc)),
                ..default()
            },
            BackgroundColor(TRACK_COLOR),
            BorderColor::all(theme::FRAME_EDGE),
        ))
        .with_children(|track| {
            track.spawn((
                Node {
                    width: Val::Percent(f32::from(pct.min(100))),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(color),
                BackgroundGradient::from(LinearGradient::to_bottom(vec![
                    ColorStop::new(color.mix(&Color::WHITE, 0.42), Val::Percent(0.0)),
                    ColorStop::new(color, Val::Percent(42.0)),
                    ColorStop::new(color.mix(&Color::BLACK, 0.40), Val::Percent(100.0)),
                ])),
            ));
        });
}

fn spawn_self(parent: &mut ChildSpawnerCommands, member: Option<&PartyMember>, sc: f32) {
    let gauges = [
        ("HP", member.map(|m| (m.hp, m.hp_pct)), HP_COLOR),
        ("MP", member.map(|m| (m.mp, m.mp_pct)), MP_COLOR),
        (
            "TP",
            member.map(|m| (m.tp, (m.tp / TP_UNITS_PER_PERCENT).min(100) as u8)),
            TP_COLOR,
        ),
    ];
    for (name, value, color) in gauges {
        parent
            .spawn((
                ClassicGaugeRow,
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(GAP_PX * sc),
                    ..default()
                },
            ))
            .with_children(|row| {
                label(row, name.into(), color, sc);
                gauge(row, value.map_or(0, |(_, pct)| pct), color, sc);
                row.spawn((
                    Text::new(value.map_or_else(|| MISSING_VALUE.into(), |(n, _)| n.to_string())),
                    style::text_font(FONT_PX * sc),
                    TextColor(theme::TEXT),
                    TextLayout::justify(Justify::Right),
                    Node {
                        min_width: Val::Px(crate::ui_font::text_width_px("99999", FONT_PX) * sc),
                        ..default()
                    },
                ));
            });
    }
}

fn spawn_party_member(
    parent: &mut ChildSpawnerCommands,
    member: &PartyMember,
    snap: &SceneSnapshot,
    own: Option<&PartyMember>,
    selected: Option<u32>,
    colors: &NameColorTable,
    sc: f32,
) {
    let is_self = Some(member.id) == snap.self_char_id;
    let available = own.is_some_and(|m| m.zone_no == member.zone_no);
    let name = if is_self {
        snap.char_name.as_ref().or(member.name.as_ref())
    } else {
        member.name.as_ref()
    }
    .cloned()
    .unwrap_or_default();
    parent
        .spawn((
            PartyRowTarget(member.id),
            Hovered::default(),
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::bottom(Val::Px(GAP_PX * sc)),
                ..default()
            },
            BackgroundColor(if selected == Some(member.id) {
                SELECTED_COLOR
            } else {
                Color::NONE
            }),
        ))
        .with_children(|entry| {
            entry
                .spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: Val::Px(GAP_PX * sc),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        ClassicText,
                        Text::new(name),
                        style::text_font(FONT_PX * sc),
                        TextColor(colors.color(if is_self { ncol::PC } else { ncol::PARTY })),
                        TextLayout {
                            linebreak: LineBreak::NoWrap,
                            ..default()
                        },
                        Node {
                            flex_shrink: 1.0,
                            min_width: Val::Px(0.0),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                    ));
                    label(row, numeric(member.hp, available), theme::TEXT, sc);
                });
            gauge(
                entry,
                if available { member.hp_pct } else { 0 },
                HP_COLOR,
                sc,
            );
            entry
                .spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|row| {
                    label(row, numeric(member.mp, available), MP_COLOR, sc);
                    label(row, numeric(member.tp, available), TP_COLOR, sc);
                });
        });
}

fn numeric(value: u32, available: bool) -> String {
    if available {
        value.to_string()
    } else {
        MISSING_VALUE.into()
    }
}

#[cfg(test)]
mod tests {
    use super::super::party_frame::{
        spawn_party_frames, update_party_frame_system, PartyFrameRoot,
    };
    use super::*;
    use crate::graphics_settings::GraphicsSettings;

    fn member(id: u32, party_no: u8) -> PartyMember {
        PartyMember {
            id,
            act_index: id as u16,
            name: Some(format!("Member{id}")),
            hp: 777,
            mp: 88,
            tp: 1234,
            hp_pct: 100,
            mp_pct: 50,
            zone_no: 1,
            main_job: 0,
            main_job_lv: 0,
            sub_job: 0,
            sub_job_lv: 0,
            is_party_leader: false,
            is_alliance_leader: false,
            party_no,
            in_mog_house: false,
        }
    }

    fn app(members: Vec<PartyMember>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<PartyFrameSettings>()
            .init_resource::<GraphicsSettings>()
            .init_resource::<NameColorTable>()
            .init_resource::<Target>()
            .insert_resource(SceneState {
                snapshot: SceneSnapshot {
                    self_char_id: Some(1),
                    char_name: Some("Selfname".into()),
                    party: members,
                    stage: Stage::InZone,
                    ..default()
                },
                ..default()
            })
            .add_systems(Startup, spawn_party_frames)
            .add_systems(
                Update,
                (update_party_frame_system, update_classic_status_system).chain(),
            );
        app.update();
        app
    }

    fn frame(app: &mut App, kind: ClassicFrame) -> Entity {
        app.world_mut()
            .query::<(Entity, &ClassicFrame)>()
            .iter(app.world())
            .find(|(_, frame)| **frame == kind)
            .unwrap()
            .0
    }

    fn labels(app: &mut App, kind: ClassicFrame) -> Vec<String> {
        let mut queue = vec![frame(app, kind)];
        let mut labels = Vec::new();
        while let Some(entity) = queue.pop() {
            if let Some(text) = app.world().get::<Text>(entity) {
                labels.push(text.0.clone());
            }
            if let Some(children) = app.world().get::<Children>(entity) {
                queue.extend(children.iter());
            }
        }
        labels
    }

    #[test]
    fn solo_has_three_labeled_gauges_and_no_custom_title() {
        let mut app = app(vec![member(1, ffxi_proto::decode::NO_PARTY)]);
        let labels = labels(&mut app, ClassicFrame::SelfStatus);
        for value in ["HP", "MP", "TP", "777", "88", "1234"] {
            assert!(labels.iter().any(|s| s == value));
        }
        assert!(!labels.iter().any(|s| s == "Solo" || s == "New"));
        let roster = frame(&mut app, ClassicFrame::Party(0));
        assert_eq!(
            app.world().get::<Node>(roster).unwrap().display,
            Display::None
        );
    }

    #[test]
    fn missing_self_data_cannot_use_another_members_vitals() {
        let mut app = app(vec![member(2, 0)]);
        let labels = labels(&mut app, ClassicFrame::SelfStatus);
        assert_eq!(
            labels
                .iter()
                .filter(|s| s.as_str() == MISSING_VALUE)
                .count(),
            3
        );
        assert!(!labels.iter().any(|s| s == "777"));
    }

    #[test]
    fn out_of_zone_values_are_hidden_and_target_ids_remain_available() {
        let mut other = member(2, 0);
        other.zone_no = 2;
        let mut app = app(vec![member(1, 0), other]);
        let labels = labels(&mut app, ClassicFrame::Party(0));
        assert!(labels.iter().any(|s| s == "Selfname"));
        assert!(labels.iter().any(|s| s == "Member2"));
        assert_eq!(
            labels
                .iter()
                .filter(|s| s.as_str() == MISSING_VALUE)
                .count(),
            3
        );
        let mut ids = app
            .world_mut()
            .query::<&PartyRowTarget>()
            .iter(app.world())
            .map(|r| r.0)
            .collect::<Vec<_>>();
        ids.sort();
        assert_eq!(ids, [1, 2]);
    }

    #[test]
    fn zoning_and_custom_rollback_do_not_leave_duplicate_panels() {
        let mut app = app(vec![member(1, 0), member(2, 0)]);
        app.world_mut().resource_mut::<SceneState>().snapshot.stage = Stage::Zoning;
        app.world_mut().resource_mut::<SceneState>().dirty = true;
        app.update();
        assert!(app
            .world_mut()
            .query_filtered::<&Node, With<ClassicFrame>>()
            .iter(app.world())
            .all(|n| n.display == Display::None));
        app.world_mut().resource_mut::<SceneState>().snapshot.stage = Stage::InZone;
        app.world_mut()
            .resource_mut::<PartyFrameSettings>()
            .classic_layout = false;
        app.update();
        assert!(app
            .world_mut()
            .query_filtered::<&Node, With<ClassicFrame>>()
            .iter(app.world())
            .all(|n| n.display == Display::None));
        assert!(app
            .world_mut()
            .query::<(&PartyFrameRoot, &Node)>()
            .iter(app.world())
            .any(|(f, n)| f.party_no == 0 && n.display == Display::Flex));
        app.world_mut()
            .resource_mut::<PartyFrameSettings>()
            .classic_layout = true;
        app.update();
        let self_frame = frame(&mut app, ClassicFrame::SelfStatus);
        assert_eq!(
            app.world().get::<Node>(self_frame).unwrap().display,
            Display::Flex
        );
        assert!(app
            .world_mut()
            .query_filtered::<&Node, With<PartyFrameRoot>>()
            .iter(app.world())
            .all(|n| n.display == Display::None));
    }
}
