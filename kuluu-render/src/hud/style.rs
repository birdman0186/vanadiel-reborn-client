//! The single style source for the entire HUD: in-game windows (menus, items,
//! shop, trade, dialog…) *and* dev/diagnostic overlays (diagnostics bar,
//! stage bar, network status, mesh debug). Everything takes colors and frames
//! from here so they cannot drift apart; the old `hud::palette` dev-overlay
//! module was folded into `theme` and removed.

use bevy::prelude::*;

/// Menus draw above the chat stack (which owns the lower-left quadrant), matching
/// the other pop-up windows (target-action / quick-action at 20, trade at 25).
pub const WINDOW_Z: i32 = 20;

/// Retail stacks the self, target, and party panels in one right-hand column, so
/// their width, inset, and gap are a shared contract, not independent tunings.
/// `hud::panel_column` owns the stacking; panels never position themselves
/// relative to each other.
pub const PANEL_WIDTH_PX: f32 = 220.0;
pub const PANEL_COLUMN_RIGHT_PX: f32 = 8.0;
/// Clearance under the column, leaving the chat stack's top edge visible.
pub const PANEL_COLUMN_BOTTOM_PX: f32 = 28.0;
pub const PANEL_COLUMN_GAP_PX: f32 = 8.0;

/// Our "blue" window theme: a translucent navy frame with a light steel-blue
/// edge, pale-blue title text, near-white body text, and a golden cursor
/// highlight on the focused row/slot. These are deliberate tunings (retail
/// ships no palette file to scrape), named here per no-magic-numbers.
pub mod theme {
    use bevy::prelude::Color;

    pub const FRAME_BG: Color = Color::srgba(0.034, 0.065, 0.14, 0.92);
    // Opaque FRAME_BG for map surfaces only: a translucent backing behind the
    // minimap/map image shimmers where the map texture is transparent, because
    // the alpha composites over the live moving 3D scene as the player walks
    // (kuluu-bi1s.1). Opaque backing = static fill, no edge flash.
    pub const MAP_BACKING: Color = Color::srgb(0.05, 0.07, 0.16);
    pub const FRAME_EDGE: Color = Color::srgb(0.56, 0.64, 0.76);
    pub const TITLE: Color = Color::srgb(0.96, 0.91, 0.80);
    pub const TEXT: Color = Color::srgb(0.96, 0.95, 0.91);
    pub const MUTED: Color = Color::srgb(0.58, 0.63, 0.74);
    pub const CURSOR: Color = Color::srgb(0.99, 0.86, 0.60);
    pub const CURSOR_BG: Color = Color::srgba(0.14, 0.31, 0.54, 0.92);
    pub const CELL_BG: Color = Color::srgba(0.10, 0.13, 0.24, 0.85);
    pub const CELL_EDGE: Color = Color::srgb(0.32, 0.38, 0.52);
    pub const DANGER: Color = Color::srgb(0.95, 0.35, 0.35);
    pub const WARN: Color = Color::srgb(1.0, 0.82, 0.30);
    pub const GOOD: Color = Color::srgb(0.45, 0.90, 0.55);
    pub const FAINT: Color = Color::srgb(0.44, 0.48, 0.58);
}

/// The row prefix marking the cursor in every list menu — "> " on the selected
/// row, two spaces otherwise — so the items, main-menu, quick-action and
/// target-action lists all indent identically.
pub fn cursor_prefix(is_cursor: bool) -> &'static str {
    if is_cursor {
        "> "
    } else {
        "  "
    }
}

pub fn text_font(size: f32) -> TextFont {
    TextFont {
        font_size: size.into(),
        ..default()
    }
}

/// A framed window panel: translucent navy fill, steel-blue 1px border,
/// column layout. Every game window's outer chrome goes through this.
pub fn window_frame() -> (
    Node,
    BackgroundColor,
    BorderColor,
    super::retail_frame::RetailWindow,
) {
    (
        Node {
            padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
            border: UiRect::all(Val::Px(1.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        },
        BackgroundColor(theme::FRAME_BG),
        BorderColor::all(theme::FRAME_EDGE),
        super::retail_frame::RetailWindow,
    )
}
