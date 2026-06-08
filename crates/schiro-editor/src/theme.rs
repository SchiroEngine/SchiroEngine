use egui::{Color32, CornerRadius, Stroke, Style, Visuals};

pub fn apply_dark_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    visuals.override_text_color = Some(Color32::from_rgb(0xCC, 0xCF, 0xD4));
    visuals.panel_fill = Color32::from_rgb(0x1A, 0x1B, 0x1E);
    visuals.window_fill = Color32::from_rgb(0x1E, 0x1F, 0x23);
    visuals.faint_bg_color = Color32::from_rgb(0x22, 0x23, 0x28);
    visuals.extreme_bg_color = Color32::from_rgb(0x10, 0x10, 0x14);
    visuals.code_bg_color = Color32::from_rgb(0x18, 0x19, 0x1D);
    visuals.window_corner_radius = CornerRadius::same(6);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 4].into(),
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(120),
    };
    visuals.hyperlink_color = Color32::from_rgb(0x5C, 0x8D, 0xE6);
    visuals.selection = egui::style::Selection {
        bg_fill: Color32::from_rgb(0x3D, 0x55, 0x8B),
        stroke: Stroke::new(1.0, Color32::from_rgb(0x4D, 0x78, 0xCC)),
    };
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(0x25, 0x26, 0x2B);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(3);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(0x2A, 0x2B, 0x30);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(3);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x35, 0x36, 0x3D);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(3);
    visuals.widgets.active.bg_fill = Color32::from_rgb(0x2D, 0x2E, 0x35);
    visuals.widgets.active.corner_radius = CornerRadius::same(3);
    visuals.widgets.open.bg_fill = Color32::from_rgb(0x25, 0x26, 0x2B);
    visuals.widgets.open.corner_radius = CornerRadius::same(3);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 4.0);
    style.spacing.button_padding = egui::vec2(12.0, 4.0);
    style.spacing.indent = 18.0;
    ctx.set_style(style);
}

pub fn accent_color() -> Color32 {
    Color32::from_rgb(0x4D, 0x78, 0xCC)
}

pub fn panel_header_bg() -> Color32 {
    Color32::from_rgb(0x22, 0x23, 0x28)
}

pub fn text_dim() -> Color32 {
    Color32::from_rgb(0x88, 0x8C, 0x94)
}

pub fn text_bright() -> Color32 {
    Color32::from_rgb(0xE8, 0xEA, 0xED)
}

pub fn faint_bg_color() -> Color32 {
    Color32::from_rgb(0x22, 0x23, 0x28)
}
