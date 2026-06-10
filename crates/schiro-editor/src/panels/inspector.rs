//! Inspector panel: displays and edits the components of the
//! currently selected entity.
//!
//! Translation, rotation and scale are exposed through egui
//! `DragValue` widgets. Changes are written back into the ECS
//! world via [`bevy_ecs::world::World::get_mut`]. The rotation
//! is edited in YXZ Euler degrees and converted to a quaternion
//! on the fly.

use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};
use schiro_ecs::components::{Name, Rotator, Transform};

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
        egui::ScrollArea::vertical().id_salt("insp_scroll").show(ui, |ui| {
            ui.add_space(2.0);
            self.draw_name_section(ui, entity);
            ui.add_space(8.0);
            self.draw_translation_section(ui, entity);
            ui.add_space(8.0);
            self.draw_rotation_section(ui, entity);
            ui.add_space(8.0);
            self.draw_scale_section(ui, entity);
            ui.add_space(8.0);
            self.draw_rotator_section(ui, entity);
        });
    }

    /// Editable name. Falls back to a generated name when the entity
    /// has no [`Name`] component.
    fn draw_name_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        let label = match self.world.get::<Name>(entity) {
            Some(n) => n.0.clone(),
            None => format!("Entity {}", entity.index()),
        };
        let mut next = label.clone();
        let response = ui.add(
            egui::TextEdit::singleline(&mut next)
                .hint_text("name")
                .font(egui::TextStyle::Body)
                .desired_width(ui.available_width()),
        );
        if response.lost_focus() && next != label {
            // Insert or update the Name component in place.
            if let Some(mut existing) = self.world.get_mut::<Name>(entity) {
                existing.0 = next;
            } else {
                self.world.entity_mut(entity).insert(Name(next));
            }
        }
    }

    /// Translation as an editable Vec3.
    fn draw_translation_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        ui.group(|ui| {
            ui.label(
                egui::RichText::new("Translation")
                    .color(crate::theme::text_dim())
                    .size(11.0),
            );
            let mut t = match self.world.get::<Transform>(entity) {
                Some(t) => *t,
                None => return,
            };
            let mut edited = t.translation;
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("X").color(crate::theme::accent_color()).size(11.5),
                );
                ui.add(
                    egui::DragValue::new(&mut edited.x)
                        .speed(0.05)
                        .max_decimals(3),
                );
            });
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Y").color(crate::theme::accent_color()).size(11.5),
                );
                ui.add(
                    egui::DragValue::new(&mut edited.y)
                        .speed(0.05)
                        .max_decimals(3),
                );
            });
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Z").color(crate::theme::accent_color()).size(11.5),
                );
                ui.add(
                    egui::DragValue::new(&mut edited.z)
                        .speed(0.05)
                        .max_decimals(3),
                );
            });
            if edited != t.translation {
                if let Some(mut existing) = self.world.get_mut::<Transform>(entity) {
                    existing.translation = edited;
                }
                t.translation = edited;
                let _ = t;
            }
        });
    }

    /// Rotation as YXZ Euler degrees, converted to a quaternion on
    /// the fly. We read the current rotation, convert to euler,
    /// expose the euler, and re-compose on change.
    fn draw_rotation_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        ui.group(|ui| {
            ui.label(
                egui::RichText::new("Rotation (deg)")
                    .color(crate::theme::text_dim())
                    .size(11.0),
            );
            let t = match self.world.get::<Transform>(entity) {
                Some(t) => *t,
                None => return,
            };
            let (yaw, pitch, roll) = t.rotation.to_euler(glam::EulerRot::YXZ);
            let mut deg = [yaw.to_degrees(), pitch.to_degrees(), roll.to_degrees()];
            let mut changed = false;
            for (label, value) in [("Y", &mut deg[0]), ("X", &mut deg[1]), ("Z", &mut deg[2])] {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(label)
                            .color(crate::theme::accent_color())
                            .size(11.5),
                    );
                    let resp = ui.add(
                        egui::DragValue::new(*value)
                            .speed(0.5)
                            .max_decimals(1)
                            .suffix("\u{00B0}"),
                    );
                    if resp.changed() {
                        changed = true;
                    }
                });
            }
            if changed {
                let new_rotation = Quat::from_euler(
                    glam::EulerRot::YXZ,
                    deg[0].to_radians(),
                    deg[1].to_radians(),
                    deg[2].to_radians(),
                );
                if let Some(mut existing) = self.world.get_mut::<Transform>(entity) {
                    existing.rotation = new_rotation;
                }
            }
        });
    }

    /// Scale as an editable Vec3.
    fn draw_scale_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        ui.group(|ui| {
            ui.label(
                egui::RichText::new("Scale")
                    .color(crate::theme::text_dim())
                    .size(11.0),
            );
            let t = match self.world.get::<Transform>(entity) {
                Some(t) => *t,
                None => return,
            };
            let mut edited = t.scale;
            let mut changed = false;
            for (label, value) in
                [("X", &mut edited.x), ("Y", &mut edited.y), ("Z", &mut edited.z)]
            {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(label)
                            .color(crate::theme::accent_color())
                            .size(11.5),
                    );
                    let resp = ui.add(
                        egui::DragValue::new(*value)
                            .speed(0.02)
                            .range(0.001..=1000.0)
                            .max_decimals(3),
                    );
                    if resp.changed() {
                        changed = true;
                    }
                });
            }
            if changed {
                if let Some(mut existing) = self.world.get_mut::<Transform>(entity) {
                    existing.scale = edited;
                }
            }
        });
    }

    /// Toggle button to add or remove the [`Rotator`] component.
    fn draw_rotator_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        let has_rotator = self.world.get::<Rotator>(entity).is_some();
        let label = if has_rotator { "Rotator: ON" } else { "Rotator: off" };
        let (rect, resp) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 24.0),
            egui::Sense::click(),
        );
        if ui.is_rect_visible(rect) {
            let fill = if has_rotator {
                crate::theme::accent_color()
            } else {
                crate::theme::faint_bg_color()
            };
            ui.painter().rect(
                rect,
                egui::CornerRadius::same(4),
                fill,
                egui::Stroke::NONE,
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                rect.left_center() + egui::vec2(8.0, 0.0),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(11.5),
                if has_rotator { crate::theme::text_bright() } else { crate::theme::text_dim() },
            );
        }
        if resp.clicked() {
            let mut em = self.world.entity_mut(entity);
            if has_rotator {
                em.remove::<Rotator>();
            } else {
                em.insert(Rotator::default());
            }
        }
    }
}
