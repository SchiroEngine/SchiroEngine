//! Color palette and egui theme inspired by Blender's dark
//! interface: deep charcoal backgrounds, orange accent for
//! selections, subtle grey borders and a flat clean hierarchy.
//!
//! [`apply_dark_theme`] pushed the entire Visuals + Style set
//! into the egui context once at startup. The remaining helpers
//! return individual colors that the rest of the editor reuses
//! for borders, separators and interaction feedback.

use egui::{Color32, CornerRadius, Stroke, Visuals};

// ---------------------------------------------------------------------------
// Base palette (Blender-style)
// ---------------------------------------------------------------------------

/// Deepest canvas, used for the viewport and empty scroll areas.
const DARKEST: Color32 = Color32::from_rgb(0x18, 0x18, 0x1A);

/// Panel background — same shade used by every side panel and the
/// property region.
const PANEL_BG: Color32 = Color32::from_rgb(0x22, 0x22, 0x26);

/// Slightly lighter surface, used for widget and group backgrounds.
const SURFACE: Color32 = Color32::from_rgb(0x2A, 0x2A, 0x2E);

/// Hover / highlight tint.
const HOVER: Color32 = Color32::from_rgb(0x36, 0x36, 0x3B);

/// Orange accent (Blender's &#34;#E5A93A&#34; desaturated a touch).
const ACCENT: Color32 = Color32::from_rgb(0xE5, 0x8E, 0x0C);

/// Desaturated accent used for thin borders.
const ACCENT_FAINT: Color32 = Color32::from_rgb(0x60, 0x3C, 0x06);

/// Bright text — used for labels and primary content.
const TEXT_BRIGHT: Color32 = Color32::from_rgb(0xE8, 0xE8, 0xEB);

/// Dimmed text — used for secondary info and placeholders.
const TEXT_DIM: Color32 = Color32::from_rgb(0x8C, 0x8C, 0x92);

/// Thin border colour.
const BORDER: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x3F);

// ---------------------------------------------------------------------------
// Theme entry point
// ---------------------------------------------------------------------------

/// Applies the Blender-inspired dark theme to the supplied egui
/// context.
///
/// The function overrides every [`Visuals`] field and tweaks the
/// global [`egui::Style`] so that every panel, button and widget
/// inherits the same base palette without per-widget setup.
pub fn apply_dark_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    // Override text.
    visuals.override_text_color = Some(TEXT_BRIGHT);

    // Panel / window backgrounds.
    visuals.panel_fill = PANEL_BG;
    visuals.window_fill = PANEL_BG;
    visuals.faint_bg_color = SURFACE;
    visuals.extreme_bg_color = DARKEST;
    visuals.code_bg_color = Color32::from_rgb(0x1C, 0x1C, 0x1F);

    // Window chrome.
    visuals.window_corner_radius = CornerRadius::same(6);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4].into(),
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(140),
    };

    // Links and selection.
    visuals.hyperlink_color = ACCENT;
    visuals.selection = egui::style::Selection {
        bg_fill: Color32::from_rgb(0x56, 0x36, 0x06),
        stroke: Stroke::new(1.0, ACCENT),
    };

    // Widgets — noninteractive, inactive, hovered, active, open.
    let w = &mut visuals.widgets;

    let corner = CornerRadius::same(3);
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

    // Global style overrides.
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 5.0);
    style.spacing.button_padding = egui::vec2(12.0, 5.0);
    style.spacing.indent = 18.0;
    style.spacing.interact_size = egui::vec2(24.0, 24.0);
    ctx.set_style(style);
}

// ---------------------------------------------------------------------------
// Reusable colour helpers — these map every non‑egui draw call to the
// palette above.
// ---------------------------------------------------------------------------

/// Accent colour used for selection outlines, active tool buttons and
/// gizmo axes.
pub fn accent_color() -> Color32 {
    ACCENT
}

/// Fainter version of the accent, useful for borders or subtle
/// separators.
pub fn accent_faint() -> Color32 {
    ACCENT_FAINT
}

/// Background used by panel headers (menu bar, toolbar, status bar).
pub fn panel_header_bg() -> Color32 {
    Color32::from_rgb(0x1E, 0x1E, 0x22)
}

/// Very dark background used for sunken areas (viewport, empty scroll
/// panes).
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
