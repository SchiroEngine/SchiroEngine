//! Inspector panel (Blender-style). Sections use plain
//! [`egui::CollapsingHeader`] without a helper closure so the
//! borrow checker stays happy.

use bevy_ecs::prelude::*;
use glam::Quat;
use schiro_ecs::components::{Name, Rotator, Transform};

use crate::app::EditorApp;

impl EditorApp {
    pub fn build_inspector_panel(&mut self, ctx: &egui::Context) {
        let frame = egui::Frame::new()
            .fill(crate::theme::panel_header_bg())
            .inner_margin(egui::Margin::same(0));

        egui::SidePanel::right("inspector_panel")
            .resizable(true)
            .default_width(300.0)
            .min_width(220.0)
            .frame(frame)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(crate::theme::panel_header_bg())
                    .inner_margin(egui::vec2(10.0, 5.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Properties")
                                .color(crate::theme::text_bright())
                                .size(11.5)
                                .strong(),
                        );
                    });
                ui.painter().hline(
                    ui.available_rect_before_wrap().x_range(),
                    ui.cursor().top(),
                    egui::Stroke::new(1.0_f32, crate::theme::border()),
                );
                ui.add_space(2.0);

                if let Some(entity) = self.selected_entity {
                    egui::ScrollArea::vertical()
                        .id_salt("insp_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            egui::Frame::new().inner_margin(egui::vec2(8.0, 6.0)).show(ui, |ui| {
                                egui::CollapsingHeader::new("Name")
                                    .default_open(true)
                                    .show(ui, |ui| self.draw_name_section(ui, entity));

                                ui.add_space(3.0);

                                egui::CollapsingHeader::new("Translation")
                                    .default_open(true)
                                    .show(ui, |ui| self.draw_translation_section(ui, entity));

                                ui.add_space(2.0);

                                egui::CollapsingHeader::new("Rotation (deg)")
                                    .default_open(true)
                                    .show(ui, |ui| self.draw_rotation_section(ui, entity));

                                ui.add_space(2.0);

                                egui::CollapsingHeader::new("Scale")
                                    .default_open(true)
                                    .show(ui, |ui| self.draw_scale_section(ui, entity));

                                ui.add_space(4.0);

                                egui::CollapsingHeader::new("Components").default_open(true).show(
                                    ui,
                                    |ui| {
                                        self.draw_rotator_section(ui, entity);
                                    },
                                );
                            });
                        });
                } else {
                    ui.add_space(16.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("No selection")
                                .color(crate::theme::text_dim())
                                .size(11.5),
                        );
                    });
                }
            });
    }

    fn draw_name_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        let label = match self.world.get::<Name>(entity) {
            Some(n) => n.0.clone(),
            None => format!("Entity {}", entity.index()),
        };
        let mut next = label.clone();
        if ui
            .add(
                egui::TextEdit::singleline(&mut next)
                    .hint_text("name")
                    .desired_width(ui.available_width()),
            )
            .lost_focus()
            && next != label
        {
            if let Some(mut n) = self.world.get_mut::<Name>(entity) {
                n.0 = next;
            } else {
                self.world.entity_mut(entity).insert(Name(next));
            }
        }
    }

    fn draw_translation_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        let Some(mut t) = self.world.get_mut::<Transform>(entity) else { return };
        let labels = ["X", "Y", "Z"];
        for i in 0..3 {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(labels[i])
                        .color(crate::theme::accent_color())
                        .size(11.0)
                        .monospace(),
                );
                ui.add(egui::DragValue::new(&mut t.translation[i]).speed(0.05).max_decimals(3));
            });
        }
    }

    fn draw_rotation_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        let Some(mut t) = self.world.get_mut::<Transform>(entity) else { return };
        let (yaw, pitch, roll) = t.rotation.to_euler(glam::EulerRot::YXZ);
        let mut deg = [yaw.to_degrees(), pitch.to_degrees(), roll.to_degrees()];
        let labels = ["Y", "X", "Z"];
        for i in 0..3 {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(labels[i])
                        .color(crate::theme::accent_color())
                        .size(11.0)
                        .monospace(),
                );
                ui.add(
                    egui::DragValue::new(&mut deg[i]).speed(0.5).max_decimals(1).suffix("\u{00B0}"),
                );
            });
        }
        t.rotation = Quat::from_euler(
            glam::EulerRot::YXZ,
            deg[0].to_radians(),
            deg[1].to_radians(),
            deg[2].to_radians(),
        );
    }

    fn draw_scale_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        let Some(mut t) = self.world.get_mut::<Transform>(entity) else { return };
        let labels = ["X", "Y", "Z"];
        for i in 0..3 {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(labels[i])
                        .color(crate::theme::accent_color())
                        .size(11.0)
                        .monospace(),
                );
                ui.add(
                    egui::DragValue::new(&mut t.scale[i])
                        .speed(0.02)
                        .range(0.001..=1000.0)
                        .max_decimals(3),
                );
            });
        }
    }

    fn draw_rotator_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        let has = self.world.get::<Rotator>(entity).is_some();
        let label = if has { "Rotator:  ON" } else { "Rotator:  off" };
        let (rect, resp) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
        if ui.is_rect_visible(rect) {
            let fill =
                if has { crate::theme::hover() } else { Color32::from_rgb(0x28, 0x28, 0x2C) };
            ui.painter().rect_filled(rect, egui::CornerRadius::same(3), fill);
            ui.painter().text(
                rect.left_center() + egui::vec2(8.0, 0.0),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(11.5),
                if has { crate::theme::text_bright() } else { crate::theme::text_dim() },
            );
        }
        if resp.clicked() {
            let mut em = self.world.entity_mut(entity);
            if has {
                em.remove::<Rotator>();
            } else {
                em.insert(Rotator::default());
            }
        }
    }
}

use egui::Color32;
