// .agents/skills/retail-observe/references/original-ui-resources-2026-11-15.md
use std::sync::Arc;

use bevy::prelude::*;
use ffxi_dat::menu_layout::{MenuLayout, MenuRecord, MenuResource};

use super::menu::{self, MainMenuPane};
use super::retail_text::{spawn_quads, WINDOW_LABEL_GROUP};
use crate::input_mode::{InputMode, MenuKind};
use crate::ui_element_atlas::{UiElementAtlas, UiElementDatRoot};

#[derive(Resource, Default)]
pub struct CommandLayout(Option<Arc<MenuLayout>>);

impl CommandLayout {
    pub fn page(&self) -> Option<&MenuLayout> {
        self.0.as_deref()
    }
}

fn visible_resource(record: &MenuRecord) -> Option<&MenuResource> {
    record.resources.iter().find(|resource| resource.tag == 0)
}

pub(super) fn record_for<'a>(
    page: &'a MenuLayout,
    group: &str,
    index: usize,
) -> Option<&'a MenuRecord> {
    page.children.iter().find(|child| {
        visible_resource(child).is_some_and(|resource| {
            resource.group == group.trim_end() && usize::from(resource.index) == index
        })
    })
}

pub(super) fn party_record(page: &MenuLayout, row: usize) -> Option<&MenuRecord> {
    // prty1, prty2, prty3 and prty7 use the same label pitch. The DAT stores children out of visual order.
    let y = i16::try_from(row).ok()?.checked_mul(16)?.checked_add(6)?;
    page.children.iter().find(|r| {
        r.y == y && visible_resource(r).is_some_and(|g| g.group == WINDOW_LABEL_GROUP.trim_end())
    })
}

pub(super) fn position(node: &mut Node, record: Option<&MenuRecord>) {
    let (mode, x, y) = match record {
        Some(record) => (
            PositionType::Absolute,
            Val::Px(f32::from(record.x)),
            Val::Px(f32::from(record.y)),
        ),
        None => (PositionType::Relative, Val::Auto, Val::Auto),
    };
    if (node.position_type, node.left, node.top) != (mode, x, y) {
        node.position_type = mode;
        node.left = x;
        node.top = y;
    }
}

pub(super) fn right_clearance(page: &MenuLayout) -> f32 {
    let edge = i32::from(page.root.width);
    let right = page
        .children
        .iter()
        .map(|child| i32::from(child.x) + i32::from(child.offset_x) + i32::from(child.width))
        .max()
        .unwrap_or(edge);
    (right - edge).max(0) as f32
}

#[derive(Resource, Default)]
pub struct CommandInset(f32);

impl CommandInset {
    pub fn extra(&self) -> f32 {
        self.0
    }
}

fn measure_command_inset(
    current: Res<CommandLayout>,
    mut inset: ResMut<CommandInset>,
    menu_target: Query<
        (&ComputedUiRenderTargetInfo, &ComputedUiTargetCamera),
        With<menu::MainMenu>,
    >,
    column: Query<
        (
            &Node,
            &ComputedNode,
            &UiGlobalTransform,
            &ComputedUiTargetCamera,
        ),
        With<super::panel_column::ColumnPanel>,
    >,
) {
    let extra =
        current
            .page()
            .zip(menu_target.single().ok())
            .map_or(0.0, |(page, (target, camera))| {
                let panels =
                    column
                        .iter()
                        .filter_map(|(node, computed, transform, panel_camera)| {
                            if node.display == Display::None
                                || panel_camera.get() != camera.get()
                                || computed.is_empty()
                            {
                                return None;
                            }
                            Some(Rect::from_center_size(
                                transform.affine().translation / target.scale_factor(),
                                computed.size() / target.scale_factor(),
                            ))
                        });
                column_clearance(page, target.logical_size().x, panels)
            });
    if inset.0 != extra {
        inset.0 = extra;
    }
}

fn column_clearance(
    page: &MenuLayout,
    viewport_width: f32,
    panels: impl Iterator<Item = Rect>,
) -> f32 {
    let padding = right_clearance(page);
    let right = viewport_width - padding;
    let top = f32::from(page.root.y);
    let width = f32::from(page.root.width);
    let nominal = Rect::new(right - width, top, right, top + f32::from(page.root.height));
    // Always compare the original on-screen position, including when the last frame moved the pane.
    let extra = panels
        .filter_map(|panel| {
            let intersection = nominal.intersect(panel);
            (intersection.width() > 0.0 && intersection.height() > 0.0)
                .then_some(viewport_width - panel.min.x + super::style::PANEL_COLUMN_GAP_PX)
        })
        .fold(0.0_f32, f32::max);
    let left_edge = page
        .children
        .iter()
        .map(|child| child.x)
        .min()
        .unwrap_or(0)
        .min(0);
    extra.min((nominal.min.x + f32::from(left_edge)).max(0.0))
}

fn prepare_layout(
    scene: Option<Res<crate::snapshot::SceneState>>,
    mode: Option<Res<InputMode>>,
    mut atlas: ResMut<UiElementAtlas>,
    root: Res<UiElementDatRoot>,
    mut current: ResMut<CommandLayout>,
) {
    let desired = match mode.as_deref() {
        Some(InputMode::Menu(stack)) => stack
            .current()
            .filter(|level| matches!(level.kind, MenuKind::Root | MenuKind::Party)),
        _ => None,
    };
    let Some(level) = desired else {
        if current.0.is_some() {
            current.0 = None;
        }
        return;
    };
    let name = if level.kind == MenuKind::Party {
        scene
            .as_ref()
            .map_or("menu    prty1", |s| menu::party_layout(&s.snapshot))
    } else {
        menu::root_layout_name(level.cursor)
    };
    let page = atlas.ensure_layout(name, &root);
    match (&current.0, &page) {
        (Some(old), Some(new)) if Arc::ptr_eq(old, new) => return,
        (None, None) => return,
        _ => {}
    }
    current.0 = page.filter(|page| {
        if level.kind == MenuKind::Party {
            return page.root.width > 0 && page.root.height > 0 && party_record(page, 0).is_some();
        }
        let (first, end) = menu::root_page_bounds(level.cursor);
        page.root.width > 0
            && page.root.height > 0
            && (first..end).all(|i| {
                menu::ROOT_LABEL_SPRITES[i]
                    .is_some_and(|id| record_for(page, WINDOW_LABEL_GROUP, id).is_some())
            })
    });
}

#[derive(Component)]
pub(super) struct OriginalBackdrop;

#[derive(Component)]
struct LayoutArt;

fn sync_decoration(
    mut commands: Commands,
    current: Res<CommandLayout>,
    pane: Query<Entity, With<MainMenuPane>>,
    old_art: Query<Entity, With<LayoutArt>>,
    root: Res<UiElementDatRoot>,
    mut atlas: ResMut<UiElementAtlas>,
    mut images: ResMut<Assets<Image>>,
    mut previous: Local<Option<(Entity, u64)>>,
) {
    let Ok(parent) = pane.single() else {
        *previous = None;
        return;
    };
    let key = current.page().map(|_| (parent, atlas.generation()));
    if !current.is_changed() && key == *previous {
        return;
    }
    *previous = key;
    for entity in &old_art {
        commands.entity(entity).despawn();
    }
    commands.entity(parent).remove::<OriginalBackdrop>();
    let Some(page) = current.page() else { return };
    let title = menu::original_title(MenuKind::Root);
    for (is_panel, record) in std::iter::once((true, &page.root))
        .chain(page.children.iter().map(|record| (false, record)))
    {
        let Some(resource) = visible_resource(record) else {
            continue;
        };
        let index = usize::from(resource.index);
        if resource.group == WINDOW_LABEL_GROUP
            || title.is_some_and(|title| {
                resource.group == title.group.trim_end() && index == title.index
            })
        {
            continue;
        }
        let Some(quads) = atlas.ensure_element(&resource.group, index, &root, &mut images) else {
            continue;
        };
        let art = spawn_quads(&mut commands, parent, quads);
        let mut node = Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            ..default()
        };
        if is_panel {
            commands.entity(parent).insert(OriginalBackdrop);
            commands.entity(art).insert(ZIndex(-2));
        } else {
            position(&mut node, Some(record));
        }
        commands.entity(art).insert((LayoutArt, node));
    }
}

pub(super) struct CommandLayoutPlugin;

impl Plugin for CommandLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommandLayout>()
            .init_resource::<CommandInset>()
            .add_systems(
                PostUpdate,
                measure_command_inset.after(bevy::ui::UiSystems::Layout),
            )
            .add_systems(Update, prepare_layout.before(menu::update_main_menu))
            .add_systems(
                PostUpdate,
                sync_decoration
                    .run_if(not(resource_exists::<super::astral::AstralCobalt>))
                    .before(super::retail_frame::OriginalFrameUpdate),
            );
    }
}
