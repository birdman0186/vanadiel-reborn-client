// .agents/skills/retail-observe/references/original-ui-resources-2026-11-15.md
use bevy::prelude::*;

use super::retail_layout::{right_clearance, CommandInset, CommandLayout, OriginalBackdrop};
use super::retail_text::spawn_quads;
use crate::nameplate_marker::FIRST_GLYPH_CODE;
use crate::ui_element_atlas::{UiElementAtlas, UiElementDatRoot, UiElementQuad};

use crate::hud::style::{self, theme};
use crate::snapshot::SceneState;

#[derive(Resource, Debug, Clone, Copy)]
pub struct NetStatusVisible(pub bool);

impl Default for NetStatusVisible {
    fn default() -> Self {
        Self(true)
    }
}

/// Fixed panel width so the menu help bar can reserve space for it exactly
/// (retail pins the S/R readout at top-right, overlapping the bar's frame).
pub const NET_PANEL_W: f32 = 122.0;

#[derive(Component)]
pub struct NetworkStatusPanel;

#[derive(Component)]
pub struct NetSendArrow;

#[derive(Component)]
pub struct NetRecvArrow;

#[derive(Component)]
pub struct NetPercentLabel;

#[derive(Component)]
pub struct NetSendLabel;

#[derive(Component)]
pub struct NetRecvLabel;

#[derive(Component)]
pub struct FallbackContent;

fn fallback_panel() -> Node {
    Node {
        position_type: PositionType::Absolute,
        top: Val::ZERO,
        right: Val::ZERO,
        width: Val::Px(NET_PANEL_W),
        padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
        border: UiRect::all(Val::Px(1.0)),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        column_gap: Val::Px(6.0),
        ..default()
    }
}

pub fn spawn_network_status(mut commands: Commands) {
    commands
        .spawn((
            crate::components::InGameEntity,
            NetworkStatusPanel,
            fallback_panel(),
            super::retail_frame::RetailWindow,
            BackgroundColor(theme::FRAME_BG),
            BorderColor::all(theme::FRAME_EDGE),
            // Above the menu help bar so the panel frames over it, as retail does.
            GlobalZIndex(style::WINDOW_Z + 1),
        ))
        .with_children(|p| {
            p.spawn((
                FallbackContent,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|col| {
                col.spawn((
                    NetSendArrow,
                    Text::new("\u{25B2}"),
                    style::text_font(9.0),
                    TextColor(health_color(100)),
                ));
                col.spawn((
                    NetRecvArrow,
                    Text::new("\u{25BC}"),
                    style::text_font(9.0),
                    TextColor(health_color(100)),
                ));
            });

            p.spawn((
                FallbackContent,
                Node {
                    width: Val::Px(PERCENT_BOX_W),
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                },
            ))
            .with_children(|b| {
                b.spawn((
                    NetPercentLabel,
                    Text::new("100%"),
                    style::text_font(13.0),
                    TextColor(health_color(100)),
                ));
            });

            p.spawn((
                FallbackContent,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
            ))
            .with_children(|col| {
                spawn_baud_row(col, "S", NetSendLabel);
                spawn_baud_row(col, "R", NetRecvLabel);
            });
        });
}

const NET_LAYOUT: &str = "menu    netstat";
const METER_GROUP: &str = "menu    netmeter";
const GLYPH_GROUP: &str = "font    fontshp ";
const GLYPHS: &[u8] = b"0123456789%SR";
const METER_PHASES: usize = 4;
const METER_LEVELS: usize = 3;
const METER_PARTS: usize = 4;
const MAX_RATE_CHARS: usize = 12;

#[derive(Component)]
pub struct NetworkArtwork;

#[derive(Component, Clone, Copy)]
pub enum NetworkSprite {
    Arrow(usize),
    Character { line: usize, column: usize },
}

struct NetworkAssets {
    glyphs: Vec<UiElementQuad>,
    meter: Vec<Vec<UiElementQuad>>,
}

impl NetworkAssets {
    fn load(
        atlas: &mut UiElementAtlas,
        root: &UiElementDatRoot,
        images: &mut Assets<Image>,
    ) -> Option<Self> {
        let glyphs = GLYPHS
            .iter()
            .map(|code| {
                let index = usize::from(code - FIRST_GLYPH_CODE);
                let quads = atlas.ensure_element(GLYPH_GROUP, index, root, images)?;
                let [glyph] = quads.as_slice() else {
                    return None;
                };
                Some(glyph.clone())
            })
            .collect::<Option<_>>()?;
        let meter = (0..METER_PHASES * METER_LEVELS)
            .map(|index| {
                atlas
                    .ensure_element(METER_GROUP, index, root, images)
                    .filter(|quads| quads.len() == METER_PARTS)
            })
            .collect::<Option<_>>()?;
        Some(Self { glyphs, meter })
    }
}

#[derive(Default)]
pub struct NetworkLayoutCache {
    source: Option<(Entity, u64)>,
    assets: Option<NetworkAssets>,
}

fn spawn_network_sprites(commands: &mut Commands, parent: Entity) {
    commands.entity(parent).with_children(|children| {
        let arrows = (0..METER_PARTS).map(NetworkSprite::Arrow);
        let characters = [4, MAX_RATE_CHARS, MAX_RATE_CHARS]
            .into_iter()
            .enumerate()
            .flat_map(|(line, count)| {
                (0..count).map(move |column| NetworkSprite::Character { line, column })
            });
        for sprite in arrows.chain(characters) {
            children.spawn((
                sprite,
                NetworkArtwork,
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::None,
                    ..default()
                },
                ImageNode::default(),
                Pickable::IGNORE,
            ));
        }
    });
}

fn meter_frame(health: u8, phase: usize) -> usize {
    let level = match health {
        76.. => 0,
        26..=75 => 1,
        _ => 2,
    };
    level * METER_PHASES + phase
}

fn network_character<'a>(
    assets: &'a NetworkAssets,
    text: &[u8],
    line: usize,
    column: usize,
    width: f32,
) -> Option<(&'a UiElementQuad, Rect)> {
    let code = text.get(column)?;
    let quad = &assets.glyphs[GLYPHS.iter().position(|item| item == code)?];
    // The retail reference puts the percentage beside the meter and stacks S above R.
    let (x, y, room) = match line {
        0 => (21.0, 10.0, 36.0),
        1 => (58.0, 4.0, width - 60.0),
        _ => (58.0, 15.0, width - 60.0),
    };
    let advance = (room / text.len() as f32).min(8.0).max(0.0);
    let scale = advance / 8.0;
    let left = x + column as f32 * advance;
    let rect = Rect::from_corners(
        Vec2::new(left, y),
        Vec2::new(left + quad.rect.width() * scale, y + quad.rect.height()),
    );
    Some((quad, rect))
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn sync_network_layout(
    mut commands: Commands,
    mut atlas: ResMut<UiElementAtlas>,
    root: Res<UiElementDatRoot>,
    mut images: ResMut<Assets<Image>>,
    command_layout: Option<Res<CommandLayout>>,
    inset: Option<Res<CommandInset>>,
    scene: Res<SceneState>,
    time: Res<Time>,
    mut panel: Query<
        (Entity, &mut Node),
        (
            With<NetworkStatusPanel>,
            Without<FallbackContent>,
            Without<NetworkSprite>,
        ),
    >,
    mut fallback: Query<&mut Node, (With<FallbackContent>, Without<NetworkSprite>)>,
    art: Query<Entity, With<NetworkArtwork>>,
    mut sprites: Query<(&NetworkSprite, &mut Node, &mut ImageNode)>,
    mut cache: Local<NetworkLayoutCache>,
) {
    let Ok((parent, mut node)) = panel.single_mut() else {
        return;
    };
    let layout = atlas.ensure_layout(NET_LAYOUT, &root);
    let key = (parent, atlas.generation());
    if cache.source != Some(key) {
        cache.source = Some(key);
        cache.assets = None;
        for entity in &art {
            commands.entity(entity).despawn();
        }
        commands.entity(parent).remove::<OriginalBackdrop>();
        let artwork = layout.as_deref().and_then(|page| {
            if page.root.width == 0 || page.root.height == 0 {
                return None;
            }
            let reference = page
                .root
                .resources
                .iter()
                .find(|resource| resource.tag == 0)?;
            let quads = atlas.ensure_element(
                &reference.group,
                usize::from(reference.index),
                &root,
                &mut images,
            )?;
            if quads.is_empty() {
                return None;
            }
            Some((
                page,
                quads,
                NetworkAssets::load(&mut atlas, &root, &mut images)?,
            ))
        });
        if let Some((layout, quads, assets)) = artwork {
            let backdrop = spawn_quads(&mut commands, parent, quads);
            commands
                .entity(backdrop)
                .insert((NetworkArtwork, ZIndex(-1)));
            commands.entity(parent).insert(OriginalBackdrop);
            spawn_network_sprites(&mut commands, parent);
            *node = Node {
                position_type: PositionType::Absolute,
                top: Val::Px(f32::from(layout.root.y)),
                width: Val::Px(f32::from(layout.root.width)),
                height: Val::Px(f32::from(layout.root.height)),
                ..default()
            };
            cache.assets = Some(assets);
        } else {
            *node = fallback_panel();
        }
        for mut item in &mut fallback {
            item.display = if cache.assets.is_some() {
                Display::None
            } else {
                Display::Flex
            };
        }
    }
    let (Some(assets), Some(layout)) = (&cache.assets, layout) else {
        return;
    };
    let active = command_layout.as_deref().and_then(CommandLayout::page);
    let first_page = if active.is_none() {
        atlas.ensure_layout(super::menu::root_layout_name(0), &root)
    } else {
        None
    };
    let right = active
        .or(first_page.as_deref())
        .map_or(0.0, right_clearance)
        + inset.as_deref().map_or(0.0, CommandInset::extra);
    if node.right != Val::Px(right) {
        node.right = Val::Px(right);
    }
    let stats = scene.snapshot.net_stats;
    let percent = stats.send_health.min(stats.recv_health);
    let lines = [
        format!("{percent}%"),
        format!("S {}", stats.send_bps),
        format!("R {}", stats.recv_bps),
    ];
    let phase = (time.elapsed_secs() * METER_PHASES as f32) as usize % METER_PHASES;
    for (sprite, mut node, mut image) in &mut sprites {
        let drawing = match *sprite {
            NetworkSprite::Arrow(index) => {
                let health = if index % 2 == 0 {
                    stats.send_health
                } else {
                    stats.recv_health
                };
                let quad = &assets.meter[meter_frame(health, phase)][index];
                Some((
                    quad,
                    Rect::from_corners(
                        quad.rect.min + Vec2::X * 2.0,
                        quad.rect.max + Vec2::X * 2.0,
                    ),
                ))
            }
            NetworkSprite::Character { line, column } => network_character(
                assets,
                lines[line].as_bytes(),
                line,
                column,
                f32::from(layout.root.width),
            ),
        };
        if let Some((quad, rect)) = drawing {
            let size = rect.size();
            if (node.display, node.left, node.top, node.width, node.height)
                != (
                    Display::Flex,
                    Val::Px(rect.min.x),
                    Val::Px(rect.min.y),
                    Val::Px(size.x),
                    Val::Px(size.y),
                )
            {
                node.display = Display::Flex;
                node.left = Val::Px(rect.min.x);
                node.top = Val::Px(rect.min.y);
                node.width = Val::Px(size.x);
                node.height = Val::Px(size.y);
            }
            if image.image != quad.image || image.color != quad.color {
                image.image = quad.image.clone();
                image.color = quad.color;
            }
        } else if node.display != Display::None {
            node.display = Display::None;
        }
    }
}

const PERCENT_BOX_W: f32 = 36.0;
const BAUD_BOX_W: f32 = 38.0;

fn spawn_baud_row<M: Component>(col: &mut ChildSpawnerCommands, prefix: &str, value_marker: M) {
    col.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(3.0),
        ..default()
    })
    .with_children(|row| {
        row.spawn((
            Text::new(prefix.to_string()),
            style::text_font(11.0),
            TextColor(theme::MUTED),
        ));
        row.spawn(Node {
            width: Val::Px(BAUD_BOX_W),
            justify_content: JustifyContent::FlexEnd,
            ..default()
        })
        .with_children(|b| {
            b.spawn((
                value_marker,
                Text::new("0"),
                style::text_font(11.0),
                TextColor(theme::TEXT),
            ));
        });
    });
}

#[allow(clippy::type_complexity)]
pub fn update_network_status(
    time: Res<Time>,
    state: Res<SceneState>,
    mut send_arrow: Query<
        &mut TextColor,
        (
            With<NetSendArrow>,
            Without<NetRecvArrow>,
            Without<NetPercentLabel>,
        ),
    >,
    mut recv_arrow: Query<
        &mut TextColor,
        (
            With<NetRecvArrow>,
            Without<NetSendArrow>,
            Without<NetPercentLabel>,
        ),
    >,
    mut percent: Query<
        (&mut Text, &mut TextColor),
        (
            With<NetPercentLabel>,
            Without<NetSendArrow>,
            Without<NetRecvArrow>,
        ),
    >,
    mut send_label: Query<
        &mut Text,
        (
            With<NetSendLabel>,
            Without<NetPercentLabel>,
            Without<NetRecvLabel>,
        ),
    >,
    mut recv_label: Query<
        &mut Text,
        (
            With<NetRecvLabel>,
            Without<NetPercentLabel>,
            Without<NetSendLabel>,
        ),
    >,
) {
    let net = state.snapshot.net_stats;
    let overall = net.send_health.min(net.recv_health);

    let t = time.elapsed_secs();

    if let Ok(mut tc) = send_arrow.single_mut() {
        tc.0 = animated_arrow_color(net.send_health, t);
    }
    if let Ok(mut tc) = recv_arrow.single_mut() {
        tc.0 = animated_arrow_color(net.recv_health, t);
    }
    if let Ok((mut text, mut tc)) = percent.single_mut() {
        let want = format!("{overall}%");
        if **text != want {
            **text = want;
        }
        tc.0 = health_color(overall);
    }
    if let Ok(mut text) = send_label.single_mut() {
        let want = net.send_bps.to_string();
        if **text != want {
            **text = want;
        }
    }
    if let Ok(mut text) = recv_label.single_mut() {
        let want = net.recv_bps.to_string();
        if **text != want {
            **text = want;
        }
    }
}

pub fn apply_net_status_visibility(
    visible: Res<NetStatusVisible>,
    mut q: Query<&mut Visibility, With<NetworkStatusPanel>>,
) {
    if !visible.is_changed() {
        return;
    }
    let want = if visible.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in q.iter_mut() {
        if *v != want {
            *v = want;
        }
    }
}

pub fn health_color(pct: u8) -> Color {
    let good = Color::srgb(0.0, 0.85, 0.0);
    let warn = Color::srgb(1.0, 0.85, 0.0);
    let bad = Color::srgb(0.95, 0.20, 0.20);
    let f = (pct as f32 / 100.0).clamp(0.0, 1.0);
    if f >= 0.5 {
        warn.mix(&good, (f - 0.5) * 2.0)
    } else {
        bad.mix(&warn, f * 2.0)
    }
}

fn animated_arrow_color(health: u8, t: f32) -> Color {
    let base = health_color(health);
    let speed = 1.5 + (health as f32 / 100.0) * 4.5;
    let pulse = 0.5 + 0.5 * (t * speed).sin();
    let alpha = 0.45 + 0.55 * pulse;
    base.with_alpha(alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(c: Color) -> (f32, f32, f32) {
        let s = c.to_srgba();
        (s.red, s.green, s.blue)
    }

    #[test]
    fn full_health_is_green() {
        let (r, g, b) = rgb(health_color(100));
        assert!(g > 0.8 && r < 0.05 && b < 0.05);
    }

    #[test]
    fn zero_health_is_red() {
        let (r, g, b) = rgb(health_color(0));
        assert!(r > 0.8 && g < 0.25 && b < 0.25);
    }

    #[test]
    fn mid_health_is_yellowish() {
        let (r, g, b) = rgb(health_color(50));
        assert!(r > 0.8 && g > 0.8 && b < 0.05);
    }

    #[test]
    fn arrow_pulse_stays_within_alpha_band() {
        for i in 0..64 {
            let t = i as f32 * 0.1;
            let a = animated_arrow_color(100, t).to_srgba().alpha;
            assert!((0.44..=1.01).contains(&a), "alpha out of band: {a}");
        }
    }
}
