//! UI helpers: Blender-inspired theme palette and entity icon
//! lookup.  Reusable by any crate that builds an egui interface
//! on top of the SchiroEngine workspace.

use egui::{Color32, CornerRadius, Stroke, Visuals};

// ---------------------------------------------------------------------------
// Base palette (Blender-style)
// ---------------------------------------------------------------------------

/// Deepest canvas, used for the viewport and empty scroll areas.
pub const DARKEST: Color32 = Color32::from_rgb(0x18, 0x18, 0x1A);

/// Panel background — same shade used by every side panel and the
/// property region.
pub const PANEL_BG: Color32 = Color32::from_rgb(0x22, 0x22, 0x26);

/// Slightly lighter surface, used for widget and group backgrounds.
pub const SURFACE: Color32 = Color32::from_rgb(0x2A, 0x2A, 0x2E);

/// Hover / highlight tint.
pub const HOVER: Color32 = Color32::from_rgb(0x36, 0x36, 0x3B);

/// Orange accent (Blender-style).
pub const ACCENT: Color32 = Color32::from_rgb(0xE5, 0x8E, 0x0C);

/// Desaturated accent used for thin borders.
pub const ACCENT_FAINT: Color32 = Color32::from_rgb(0x60, 0x3C, 0x06);

/// Bright text — used for labels and primary content.
pub const TEXT_BRIGHT: Color32 = Color32::from_rgb(0xE8, 0xE8, 0xEB);

/// Dimmed text — used for secondary info and placeholders.
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x8C, 0x8C, 0x92);

/// Thin border colour.
pub const BORDER: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x3F);

// ---------------------------------------------------------------------------
// Theme entry point
// ---------------------------------------------------------------------------

/// Applies the Blender-inspired dark theme to the supplied egui
/// context.
pub fn apply_dark_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(TEXT_BRIGHT);
    visuals.panel_fill = PANEL_BG;
    visuals.window_fill = PANEL_BG;
    visuals.faint_bg_color = SURFACE;
    visuals.extreme_bg_color = DARKEST;
    visuals.code_bg_color = Color32::from_rgb(0x1C, 0x1C, 0x1F);

    visuals.window_corner_radius = CornerRadius::same(6);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4].into(),
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(140),
    };

    visuals.hyperlink_color = ACCENT;
    visuals.selection = egui::style::Selection {
        bg_fill: Color32::from_rgb(0x56, 0x36, 0x06),
        stroke: Stroke::new(1.0_f32, ACCENT),
    };

    let corner = CornerRadius::same(3);
    let w = &mut visuals.widgets;

    w.noninteractive.bg_fill = Color32::from_rgb(0x28, 0x28, 0x2C);
    w.noninteractive.corner_radius = corner;

    w.inactive.bg_fill = Color32::from_rgb(0x2E, 0x2E, 0x33);
    w.inactive.weak_bg_fill = Color32::from_rgb(0x28, 0x28, 0x2C);
    w.inactive.corner_radius = corner;

    w.hovered.bg_fill = HOVER;
    w.hovered.weak_bg_fill = Color32::from_rgb(0x30, 0x30, 0x35);
    w.hovered.corner_radius = corner;

    w.active.bg_fill = Color32::from_rgb(0x35, 0x35, 0x3A);
    w.active.weak_bg_fill = Color32::from_rgb(0x30, 0x30, 0x35);
    w.active.corner_radius = corner;

    w.open.bg_fill = Color32::from_rgb(0x28, 0x28, 0x2C);
    w.open.weak_bg_fill = Color32::from_rgb(0x28, 0x28, 0x2C);
    w.open.corner_radius = corner;

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 5.0);
    style.spacing.button_padding = egui::vec2(12.0, 5.0);
    style.spacing.indent = 18.0;
    style.spacing.interact_size = egui::vec2(24.0, 24.0);
    ctx.set_style(style);
}

// ---------------------------------------------------------------------------
// Reusable colour helpers
// ---------------------------------------------------------------------------

/// Accent colour used for selection outlines, active tool buttons and
/// gizmo axes.
pub fn accent_color() -> Color32 {
    ACCENT
}

/// Fainter version of the accent, useful for separators.
pub fn accent_faint() -> Color32 {
    ACCENT_FAINT
}

/// Background used by panel headers (menu bar, toolbar, status bar).
pub fn panel_header_bg() -> Color32 {
    Color32::from_rgb(0x1E, 0x1E, 0x22)
}

/// Very dark background used for sunken areas.
pub fn darkest() -> Color32 {
    DARKEST
}

/// Bright text — labels and primary content.
pub fn text_bright() -> Color32 {
    TEXT_BRIGHT
}

/// Dimmed text — secondary info and placeholders.
pub fn text_dim() -> Color32 {
    TEXT_DIM
}

/// Thin border colour.
pub fn border() -> Color32 {
    BORDER
}

/// Hover highlight.
pub fn hover() -> Color32 {
    HOVER
}

/// Surface fill (used for inspector groups and hierarchy rows).
pub fn surface() -> Color32 {
    SURFACE
}

/// Light surface used when a panel background is slightly elevated.
pub fn faint_bg_color() -> Color32 {
    SURFACE
}

// ---------------------------------------------------------------------------
// Entity icons
// ---------------------------------------------------------------------------

/// Returns a Unicode glyph that visually represents the entity type.
pub fn entity_icon(name: &str) -> &'static str {
    if name.contains("Cube") {
        "\u{25A0}"
    } else if name.contains("Sphere") {
        "\u{25C9}"
    } else if name.contains("Plane") {
        "\u{25A1}"
    } else if name.contains("Light") {
        "\u{2606}"
    } else {
        "\u{25CB}"
    }
}
