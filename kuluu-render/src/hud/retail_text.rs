use bevy::prelude::*;

use crate::ui_element_atlas::{UiElementAtlas, UiElementDatRoot, UiElementQuad};

pub const WINDOW_LABEL_GROUP: &str = "menu    windowps";
pub const COMMAND_CURSOR_GROUP: &str = "anc     anc_s   ";

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetailMenuLabel {
    pub group: &'static str,
    pub index: usize,
}

impl RetailMenuLabel {
    pub const fn new(group: &'static str, index: usize) -> Self {
        Self { group, index }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetailMenuCursor {
    pub group: &'static str,
    pub index: usize,
    pub visible: bool,
}

impl RetailMenuCursor {
    pub const fn command(visible: bool) -> Self {
        Self {
            group: COMMAND_CURSOR_GROUP,
            index: 0,
            visible,
        }
    }
}

#[derive(Component)]
struct LabelArt {
    label: RetailMenuLabel,
    root: Entity,
    fallback_color: Color,
    fallback_layout: Option<LabelLayout>,
    generation: u64,
}

#[derive(Clone, Copy)]
struct LabelLayout {
    width: Val,
    height: Val,
    min_width: Val,
    min_height: Val,
    flex_shrink: f32,
}

impl LabelLayout {
    fn read(node: &Node) -> Self {
        Self {
            width: node.width,
            height: node.height,
            min_width: node.min_width,
            min_height: node.min_height,
            flex_shrink: node.flex_shrink,
        }
    }

    fn restore(self, node: &mut Node) {
        node.width = self.width;
        node.height = self.height;
        node.min_width = self.min_width;
        node.min_height = self.min_height;
        node.flex_shrink = self.flex_shrink;
    }
}

fn apply_authored_layout(node: &mut Node, bounds: Rect) {
    node.width = Val::Px(bounds.width());
    node.height = Val::Px(bounds.height());
    node.min_width = node.width;
    node.min_height = node.height;
    node.flex_shrink = 0.0;
}

#[derive(Component)]
struct CursorArt {
    cursor: RetailMenuCursor,
    root: Entity,
    generation: u64,
}

pub fn authored_bounds(quads: &[UiElementQuad]) -> Option<Rect> {
    if quads.is_empty()
        || quads
            .iter()
            .any(|quad| !quad.rect.min.is_finite() || !quad.rect.max.is_finite())
    {
        return None;
    }
    Some(Rect::from_corners(
        quads
            .iter()
            .map(|quad| quad.rect.min)
            .fold(Vec2::ZERO, Vec2::min),
        quads
            .iter()
            .map(|quad| quad.rect.max)
            .fold(Vec2::ZERO, Vec2::max),
    ))
}

#[derive(Component)]
struct OriginalQuadColor(Color);

pub(super) fn spawn_quads(
    commands: &mut Commands,
    parent: Entity,
    quads: Vec<UiElementQuad>,
) -> Entity {
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: Val::ZERO,
                ..default()
            },
            bevy::picking::Pickable::IGNORE,
        ))
        .with_children(|art| {
            for quad in quads {
                art.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(quad.rect.min.x),
                        top: Val::Px(quad.rect.min.y),
                        width: Val::Px(quad.rect.width()),
                        height: Val::Px(quad.rect.height()),
                        ..default()
                    },
                    OriginalQuadColor(quad.color),
                    ImageNode {
                        color: quad.color,
                        ..ImageNode::new(quad.image)
                    },
                    bevy::picking::Pickable::IGNORE,
                ));
            }
        })
        .id();
    commands.entity(parent).add_child(root);
    root
}

fn sync_labels(
    mut commands: Commands,
    mut atlas: ResMut<UiElementAtlas>,
    dat_root: Res<UiElementDatRoot>,
    mut images: ResMut<Assets<Image>>,
    mut labels: Query<(
        Entity,
        &RetailMenuLabel,
        Option<&LabelArt>,
        Option<&mut TextColor>,
        Option<&mut Node>,
        Has<super::menu::MainMenuRow>,
    )>,
) {
    for (entity, label, loaded, color, mut node, is_row) in &mut labels {
        let fixed_box = is_row
            && node
                .as_ref()
                .is_some_and(|n| n.position_type == PositionType::Absolute);
        if loaded
            .is_some_and(|loaded| loaded.label == *label && loaded.generation == atlas.generation())
            && !dat_root.is_changed()
        {
            continue;
        }
        let fallback_color = loaded
            .map(|loaded| loaded.fallback_color)
            .or_else(|| color.as_ref().map(|color| color.0))
            .unwrap_or(Color::WHITE);
        let fallback_layout = if fixed_box {
            None
        } else {
            loaded
                .and_then(|loaded| loaded.fallback_layout)
                .or_else(|| node.as_ref().map(|node| LabelLayout::read(node)))
        };
        if let Some(loaded) = loaded {
            commands.entity(loaded.root).despawn();
            commands.entity(entity).remove::<LabelArt>();
            if let Some(mut color) = color {
                if color.0 == Color::NONE {
                    color.0 = fallback_color;
                }
            }
            if let (Some(layout), Some(node)) = (
                loaded.fallback_layout.filter(|_| !fixed_box),
                node.as_deref_mut(),
            ) {
                layout.restore(node);
            }
        }
        let Some(quads) = atlas.ensure_element(label.group, label.index, &dat_root, &mut images)
        else {
            continue;
        };
        let Some(bounds) = authored_bounds(&quads) else {
            continue;
        };
        if let Some(node) = node.as_deref_mut().filter(|_| !fixed_box) {
            apply_authored_layout(node, bounds);
        }
        let root = spawn_quads(&mut commands, entity, quads);
        commands.entity(entity).insert(LabelArt {
            label: *label,
            root,
            fallback_color,
            fallback_layout,
            generation: atlas.generation(),
        });
    }
}

fn sync_cursors(
    mut commands: Commands,
    mut atlas: ResMut<UiElementAtlas>,
    dat_root: Res<UiElementDatRoot>,
    mut images: ResMut<Assets<Image>>,
    cursors: Query<(
        Entity,
        &RetailMenuCursor,
        Option<&CursorArt>,
        Has<LabelArt>,
        Option<&TextFont>,
    )>,
) {
    for (entity, cursor, loaded, has_art_label, font) in &cursors {
        if loaded.is_some_and(|loaded| {
            has_art_label && loaded.cursor == *cursor && loaded.generation == atlas.generation()
        }) && !dat_root.is_changed()
        {
            continue;
        }
        if let Some(loaded) = loaded {
            commands.entity(loaded.root).despawn();
            commands.entity(entity).remove::<CursorArt>();
        }
        if !cursor.visible || !has_art_label {
            continue;
        }
        let quads = atlas
            .ensure_element(cursor.group, cursor.index, &dat_root, &mut images)
            .filter(|quads| authored_bounds(quads).is_some());
        let root = if let Some(quads) = quads {
            spawn_quads(&mut commands, entity, quads)
        } else {
            commands
                .spawn((
                    Text::new(">"),
                    font.cloned().unwrap_or_default(),
                    TextColor(super::style::theme::CURSOR),
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Percent(100.0),
                        top: Val::ZERO,
                        ..default()
                    },
                    ChildOf(entity),
                    bevy::picking::Pickable::IGNORE,
                ))
                .id()
        };
        commands.entity(entity).insert(CursorArt {
            cursor: *cursor,
            root,
            generation: atlas.generation(),
        });
    }
}

// .agents/skills/retail-observe/references/original-ui-resources-2026-11-15.md
fn apply_command_focus(
    rows: Query<(&RetailMenuCursor, &LabelArt), Or<(Changed<RetailMenuCursor>, Added<LabelArt>)>>,
    children: Query<&Children>,
    mut parts: Query<(&OriginalQuadColor, &mut ImageNode)>,
) {
    for (cursor, label) in &rows {
        let Ok(quads) = children.get(label.root) else {
            continue;
        };
        for child in quads {
            let Ok((base, mut image)) = parts.get_mut(*child) else {
                continue;
            };
            image.color = if cursor.visible {
                let color = base.0.to_srgba();
                let light = color.red.max(color.green).max(color.blue);
                let (green, blue) = if color.blue > color.red {
                    (0.654, 0.386)
                } else {
                    (0.765, 0.364)
                };
                Color::srgba(light, light * green, light * blue, color.alpha)
            } else {
                base.0
            };
        }
    }
}

fn hide_fallback_text(mut labels: Query<(&mut TextColor, &mut LabelArt), With<RetailMenuLabel>>) {
    for (mut color, mut art) in &mut labels {
        if color.0 != Color::NONE {
            art.fallback_color = color.0;
            color.0 = Color::NONE;
        }
    }
}

fn clear_removed_art(
    mut commands: Commands,
    mut removed_labels: RemovedComponents<RetailMenuLabel>,
    mut removed_cursors: RemovedComponents<RetailMenuCursor>,
    mut labels: Query<(&LabelArt, Option<&mut TextColor>, Option<&mut Node>)>,
    cursors: Query<&CursorArt>,
) {
    for entity in removed_labels.read() {
        let Ok((art, color, node)) = labels.get_mut(entity) else {
            continue;
        };
        commands.entity(art.root).despawn();
        commands.entity(entity).remove::<LabelArt>();
        if let Some(mut color) = color {
            if color.0 == Color::NONE {
                color.0 = art.fallback_color;
            }
        }
        if let (Some(layout), Some(mut node)) = (art.fallback_layout, node) {
            layout.restore(&mut node);
        }
    }
    for entity in removed_cursors.read() {
        if let Ok(art) = cursors.get(entity) {
            commands.entity(art.root).despawn();
            commands.entity(entity).remove::<CursorArt>();
        }
    }
}

pub struct RetailMenuLabelsPlugin;

impl Plugin for RetailMenuLabelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(super::retail_layout::CommandLayoutPlugin);
        app.add_systems(
            PostUpdate,
            (
                clear_removed_art,
                sync_labels,
                sync_cursors,
                apply_command_focus,
                hide_fallback_text,
            )
                .chain()
                .after(super::retail_frame::OriginalFrameUpdate)
                .before(bevy::ui::UiSystems::Prepare),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad(rect: Rect, color: Color) -> UiElementQuad {
        UiElementQuad {
            image: Handle::default(),
            rect,
            color,
        }
    }

    #[test]
    fn authored_bounds_keep_the_original_origin_and_negative_cursor_offset() {
        let letter = quad(Rect::new(7.0, 3.0, 16.0, 14.0), Color::WHITE);
        assert_eq!(
            authored_bounds(&[letter]),
            Some(Rect::new(0.0, 0.0, 16.0, 14.0))
        );
        let cursor = quad(Rect::new(-32.0, -2.0, 0.0, 18.0), Color::WHITE);
        assert_eq!(
            authored_bounds(&[cursor]),
            Some(Rect::new(-32.0, -2.0, 0.0, 18.0))
        );
        assert!(authored_bounds(&[]).is_none());
    }

    #[test]
    fn quad_nodes_keep_dat_positions_colors_and_order() {
        let mut app = App::new();
        let parent = app.world_mut().spawn(Node::default()).id();
        app.add_systems(
            Update,
            move |mut commands: Commands, mut ran: Local<bool>| {
                if *ran {
                    return;
                }
                *ran = true;
                spawn_quads(
                    &mut commands,
                    parent,
                    vec![
                        quad(Rect::new(7.0, 3.0, 16.0, 14.0), Color::BLACK),
                        quad(Rect::new(5.0, 1.0, 14.0, 12.0), Color::WHITE),
                    ],
                );
            },
        );
        app.update();
        let world = app.world();
        let art = world.get::<Children>(parent).unwrap()[0];
        let children = world.get::<Children>(art).unwrap();
        assert_eq!(children.len(), 2);
        for (child, expected) in children
            .iter()
            .zip([(7.0, 3.0, Color::BLACK), (5.0, 1.0, Color::WHITE)])
        {
            let node = world.get::<Node>(child).unwrap();
            assert_eq!(
                (node.left, node.top),
                (Val::Px(expected.0), Val::Px(expected.1))
            );
            assert_eq!((node.width, node.height), (Val::Px(9.0), Val::Px(11.0)));
            assert_eq!(world.get::<ImageNode>(child).unwrap().color, expected.2);
            assert_eq!(
                *world.get::<bevy::picking::Pickable>(child).unwrap(),
                bevy::picking::Pickable::IGNORE
            );
        }
    }

    #[test]
    fn unavailable_art_keeps_the_semantic_text_visible() {
        let mut app = App::new();
        app.init_resource::<UiElementAtlas>()
            .init_resource::<UiElementDatRoot>()
            .init_resource::<Assets<Image>>()
            .add_plugins(RetailMenuLabelsPlugin);
        let entity = app
            .world_mut()
            .spawn((
                Text::new("Status"),
                TextColor(Color::WHITE),
                RetailMenuLabel::new(WINDOW_LABEL_GROUP, 13),
            ))
            .id();
        app.update();
        assert_eq!(app.world().get::<Text>(entity).unwrap().0, "Status");
        assert_eq!(
            app.world().get::<TextColor>(entity).unwrap().0,
            Color::WHITE
        );
        assert!(app.world().get::<LabelArt>(entity).is_none());
    }

    #[test]
    fn replacing_a_label_with_missing_art_restores_text_and_removes_old_art() {
        let mut app = App::new();
        app.init_resource::<UiElementAtlas>()
            .init_resource::<UiElementDatRoot>()
            .init_resource::<Assets<Image>>()
            .add_plugins(RetailMenuLabelsPlugin);
        let old_art = app.world_mut().spawn(Node::default()).id();
        let entity = app
            .world_mut()
            .spawn((
                Text::new("Fallback"),
                TextColor(Color::NONE),
                RetailMenuLabel::new("missing", 0),
                LabelArt {
                    label: RetailMenuLabel::new(WINDOW_LABEL_GROUP, 13),
                    root: old_art,
                    fallback_color: Color::WHITE,
                    fallback_layout: None,
                    generation: 0,
                },
            ))
            .id();
        app.update();
        assert!(app.world().get_entity(old_art).is_err());
        assert_eq!(
            app.world().get::<TextColor>(entity).unwrap().0,
            Color::WHITE
        );
        assert!(app.world().get::<LabelArt>(entity).is_none());
    }

    #[test]
    fn removing_the_marker_restores_fallback_without_changing_the_text_or_layout() {
        let mut app = App::new();
        app.init_resource::<UiElementAtlas>()
            .init_resource::<UiElementDatRoot>()
            .init_resource::<Assets<Image>>()
            .add_plugins(RetailMenuLabelsPlugin);
        let old_art = app.world_mut().spawn(Node::default()).id();
        let node = Node {
            width: Val::Percent(100.0),
            ..default()
        };
        let mut art_node = node.clone();
        apply_authored_layout(&mut art_node, Rect::new(0.0, 0.0, 88.0, 16.0));
        assert_eq!(art_node.height, Val::Px(16.0));
        let entity = app
            .world_mut()
            .spawn((
                Text::new("Status"),
                art_node,
                TextColor(Color::NONE),
                RetailMenuLabel::new(WINDOW_LABEL_GROUP, 13),
                LabelArt {
                    label: RetailMenuLabel::new(WINDOW_LABEL_GROUP, 13),
                    root: old_art,
                    fallback_color: Color::WHITE,
                    fallback_layout: Some(LabelLayout::read(&node)),
                    generation: 0,
                },
            ))
            .id();
        app.world_mut()
            .entity_mut(entity)
            .remove::<RetailMenuLabel>();
        app.update();
        assert!(app.world().get_entity(old_art).is_err());
        assert_eq!(
            app.world().get::<TextColor>(entity).unwrap().0,
            Color::WHITE
        );
        assert_eq!(app.world().get::<Text>(entity).unwrap().0, "Status");
        assert_eq!(*app.world().get::<Node>(entity).unwrap(), node);
    }

    #[test]
    fn missing_cursor_sprite_keeps_selection_visible_on_an_art_label() {
        let mut app = App::new();
        app.init_resource::<UiElementAtlas>()
            .init_resource::<UiElementDatRoot>()
            .init_resource::<Assets<Image>>()
            .add_systems(Update, sync_cursors);
        let label_root = app.world_mut().spawn_empty().id();
        let entity = app
            .world_mut()
            .spawn((
                Text::new("> Status"),
                LabelArt {
                    label: RetailMenuLabel::new(WINDOW_LABEL_GROUP, 13),
                    root: label_root,
                    fallback_color: Color::WHITE,
                    fallback_layout: None,
                    generation: 0,
                },
                RetailMenuCursor::command(true),
            ))
            .id();
        app.update();
        let root = app.world().get::<CursorArt>(entity).unwrap().root;
        assert_eq!(app.world().get::<Text>(root).unwrap().0, ">");
        app.update();
        assert_eq!(app.world().get::<CursorArt>(entity).unwrap().root, root);
        app.world_mut().entity_mut(entity).remove::<LabelArt>();
        app.update();
        assert!(app.world().get::<CursorArt>(entity).is_none());
        assert!(app.world().get_entity(root).is_err());
    }

    #[test]
    fn real_dat_original_status_label_recovers_after_late_install_selection() {
        const STATUS_PINNED_COMPONENT_COUNT: usize = 14;
        let Some(root) = ffxi_dat::archive::open_test_install() else {
            return;
        };
        let mut app = App::new();
        app.init_resource::<UiElementAtlas>()
            .init_resource::<UiElementDatRoot>()
            .init_resource::<Assets<Image>>()
            .add_plugins(RetailMenuLabelsPlugin);
        let fallback_node = Node {
            width: Val::Percent(100.0),
            ..default()
        };
        let entity = app
            .world_mut()
            .spawn((
                Text::new("Status"),
                fallback_node.clone(),
                TextColor(Color::WHITE),
                RetailMenuLabel::new(WINDOW_LABEL_GROUP, 13),
                RetailMenuCursor::command(true),
            ))
            .id();
        app.update();
        assert!(app.world().get::<LabelArt>(entity).is_none());
        assert_eq!(
            app.world().get::<TextColor>(entity).unwrap().0,
            Color::WHITE
        );
        app.insert_resource(UiElementDatRoot(Some(std::sync::Arc::new(root))));
        app.update();
        let world = app.world();
        let art = world.get::<LabelArt>(entity).expect("original label art");
        let children = world.get::<Children>(art.root).unwrap();
        assert_eq!(children.len(), STATUS_PINNED_COMPONENT_COUNT);
        assert_eq!(world.get::<Text>(entity).unwrap().0, "Status");
        assert_eq!(world.get::<TextColor>(entity).unwrap().0, Color::NONE);
        let label_node = world.get::<Node>(entity).unwrap();
        assert_eq!(
            (label_node.width, label_node.height),
            (Val::Px(88.0), Val::Px(16.0))
        );
        assert_eq!(
            (label_node.min_height, label_node.flex_shrink),
            (Val::Px(16.0), 0.0)
        );
        let cursor = world
            .get::<CursorArt>(entity)
            .expect("original command cursor");
        let cursor_quad = world.get::<Children>(cursor.root).unwrap()[0];
        let node = world.get::<Node>(cursor_quad).unwrap();
        assert_eq!((node.left, node.top), (Val::Px(-32.0), Val::Px(-2.0)));

        let old_label_art = art.root;
        let old_cursor_art = cursor.root;
        app.world_mut().resource_mut::<UiElementDatRoot>().0 = None;
        app.update();
        assert!(app.world().get_entity(old_label_art).is_err());
        assert!(app.world().get_entity(old_cursor_art).is_err());
        assert!(app.world().get::<LabelArt>(entity).is_none());
        assert!(app.world().get::<CursorArt>(entity).is_none());
        assert_eq!(*app.world().get::<Node>(entity).unwrap(), fallback_node);
        assert_eq!(
            app.world().get::<TextColor>(entity).unwrap().0,
            Color::WHITE
        );
    }
}
