//! Inspector panel: displays the components of the currently selected
//! entity in a read only fashion.

use bevy_ecs::prelude::*;
use glam::Vec3;
use schiro_ecs::components::{Rotator, Transform};

use crate::app::EditorApp;

impl EditorApp {
    /// Builds the right hand inspector panel.
    pub fn build_inspector_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector_panel")
            .resizable(true)
            .default_width(300.0)
            .min_width(220.0)
            .frame(
                egui::Frame::none()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::vec2(8.0, 6.0)),
            )
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Inspector")
                        .color(crate::theme::text_bright())
                        .size(13.0)
                        .strong(),
                );
                ui.add_space(6.0);
                if let Some(entity) = self.selected_entity {
                    self.draw_inspector_content(ui, entity);
                } else {
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("No object selected")
                            .color(crate::theme::text_dim())
                            .size(12.0),
                    );
                }
            });
    }

    /// Draws the body of the inspector for a single entity.
    fn draw_inspector_content(&mut self, ui: &mut egui::Ui, entity: Entity) {
        let name = self.get_entity_name(entity);
        let t = self.get_entity_transform(entity);
        egui::ScrollArea::vertical().id_salt("insp_scroll").show(ui, |ui| {
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(&name).color(crate::theme::text_bright()).size(14.0).strong(),
            );
            ui.add_space(8.0);
            self.draw_transform_section(ui, &t);
            ui.add_space(8.0);
            self.draw_rotation_section(ui, &t);
            ui.add_space(8.0);
            let has_rotator = self.world.get::<Rotator>(entity).is_some();
            ui.label(format!("Rotator: {}", if has_rotator { "ON" } else { "off" }));
        });
    }

    /// Draws the read only transform section of the inspector.
    fn draw_transform_section(&self, ui: &mut egui::Ui, t: &Transform) {
        let bg = crate::theme::faint_bg_color();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 60.0), egui::Sense::hover());
        ui.painter().rect(
            rect,
            egui::CornerRadius::same(4),
            bg,
            egui::Stroke::NONE,
            egui::StrokeKind::Inside,
        );
        let cr = rect.shrink(8.0);
        let mut y = cr.min.y + 4.0;
        ui.painter().text(
            egui::pos2(cr.min.x, y),
            egui::Align2::LEFT_TOP,
            "Transform",
            egui::FontId::proportional(11.0),
            crate::theme::text_dim(),
        );
        y += 16.0;
        for (label, val) in [("X", t.translation.x), ("Y", t.translation.y), ("Z", t.translation.z)]
        {
            ui.painter().text(
                egui::pos2(cr.min.x, y),
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::proportional(11.5),
                crate::theme::accent_color(),
            );
            ui.painter().text(
                egui::pos2(cr.min.x + 18.0, y),
                egui::Align2::LEFT_TOP,
                format!("{:.3}", val),
                egui::FontId::monospace(12.0),
                crate::theme::text_bright(),
            );
            y += 17.0;
        }
    }

    /// Draws the read only rotation section of the inspector.
    fn draw_rotation_section(&self, ui: &mut egui::Ui, t: &Transform) {
        let bg = crate::theme::faint_bg_color();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 40.0), egui::Sense::hover());
        ui.painter().rect(
            rect,
            egui::CornerRadius::same(4),
            bg,
            egui::Stroke::NONE,
            egui::StrokeKind::Inside,
        );
        let cr = rect.shrink(8.0);
        let mut y = cr.min.y + 4.0;
        ui.painter().text(
            egui::pos2(cr.min.x, y),
            egui::Align2::LEFT_TOP,
            "Rotation",
            egui::FontId::proportional(11.0),
            crate::theme::text_dim(),
        );
        y += 16.0;
        let euler = t.rotation.to_euler(glam::EulerRot::YXZ);
        ui.painter().text(
            egui::pos2(cr.min.x, y),
            egui::Align2::LEFT_TOP,
            format!(
                "{:.1}\u{00B0}, {:.1}\u{00B0}, {:.1}\u{00B0}",
                euler.0.to_degrees(),
                euler.1.to_degrees(),
                euler.2.to_degrees()
            ),
            egui::FontId::monospace(11.5),
            crate::theme::text_bright(),
        );
    }
}
