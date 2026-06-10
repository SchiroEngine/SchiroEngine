//! Inspector panel (Blender-style). Records undo commands on
//! every mutation.

use bevy_ecs::prelude::*;
use glam::Quat;
use schiro_ecs::components::{Name, Rotator, Transform};

use crate::app::EditorApp;
use crate::command::Command;

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
        let before = self.get_entity_name(entity);
        let mut next = before.clone();
        if ui
            .add(
                egui::TextEdit::singleline(&mut next)
                    .hint_text("name")
                    .desired_width(ui.available_width()),
            )
            .lost_focus()
            && next != before
        {
            if let Some(mut n) = self.world.get_mut::<Name>(entity) {
                let old = n.0.clone();
                n.0 = next.clone();
                self.push_command(Command::SetName { entity, before: old, after: next });
            } else {
                self.world.entity_mut(entity).insert(Name(next.clone()));
                self.push_command(Command::SetName { entity, before: String::new(), after: next });
            }
        }
    }

    fn draw_translation_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        let start = match self.world.get::<Transform>(entity) {
            Some(t) => (t.translation, t.rotation, t.scale),
            None => return,
        };
        let labels = ["X", "Y", "Z"];
        for i in 0..3 {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(labels[i])
                        .color(crate::theme::accent_color())
                        .size(11.0)
                        .monospace(),
                );
                if let Some(mut t) = self.world.get_mut::<Transform>(entity) {
                    ui.add(egui::DragValue::new(&mut t.translation[i]).speed(0.05).max_decimals(3));
                }
            });
        }
        let end = match self.world.get::<Transform>(entity) {
            Some(t) => (t.translation, t.rotation, t.scale),
            None => return,
        };
        if end != start {
            self.push_command(Command::SetTransform { entity, before: start, after: end });
        }
    }

    fn draw_rotation_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        let start = match self.world.get::<Transform>(entity) {
            Some(t) => (t.translation, t.rotation, t.scale),
            None => return,
        };
        let (yaw, pitch, roll) = start.1.to_euler(glam::EulerRot::YXZ);
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
        if let Some(mut t) = self.world.get_mut::<Transform>(entity) {
            t.rotation = Quat::from_euler(
                glam::EulerRot::YXZ,
                deg[0].to_radians(),
                deg[1].to_radians(),
                deg[2].to_radians(),
            );
        }
        let end = match self.world.get::<Transform>(entity) {
            Some(t) => (t.translation, t.rotation, t.scale),
            None => return,
        };
        if end != start {
            self.push_command(Command::SetTransform { entity, before: start, after: end });
        }
    }

    fn draw_scale_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        let start = match self.world.get::<Transform>(entity) {
            Some(t) => (t.translation, t.rotation, t.scale),
            None => return,
        };
        let labels = ["X", "Y", "Z"];
        for i in 0..3 {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(labels[i])
                        .color(crate::theme::accent_color())
                        .size(11.0)
                        .monospace(),
                );
                if let Some(mut t) = self.world.get_mut::<Transform>(entity) {
                    ui.add(
                        egui::DragValue::new(&mut t.scale[i])
                            .speed(0.02)
                            .range(0.001..=1000.0)
                            .max_decimals(3),
                    );
                }
            });
        }
        let end = match self.world.get::<Transform>(entity) {
            Some(t) => (t.translation, t.rotation, t.scale),
            None => return,
        };
        if end != start {
            self.push_command(Command::SetTransform { entity, before: start, after: end });
        }
    }

    fn draw_rotator_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        let has = self.world.get::<Rotator>(entity).is_some();
        let label = if has { "Rotator:  ON" } else { "Rotator:  off" };
        let (rect, resp) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
        if ui.is_rect_visible(rect) {
            let fill =
                if has { crate::theme::hover() } else { egui::Color32::from_rgb(0x28, 0x28, 0x2C) };
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
                let speed = em.get::<Rotator>().map(|r| r.speed).unwrap_or(glam::Vec3::ONE);
                em.remove::<Rotator>();
                self.push_command(Command::ToggleRotator { entity, added: false, speed });
            } else {
                em.insert(Rotator::default());
                self.push_command(Command::ToggleRotator {
                    entity,
                    added: true,
                    speed: glam::Vec3::ONE,
                });
            }
        }
    }
}
