//! Hierarchy panel with painter-based entity icons.

use crate::app::EditorApp;
use crate::icons::{draw_icon, icon_for_entity, Icon};

impl EditorApp {
    pub fn build_hierarchy_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("hierarchy_panel")
            .resizable(true)
            .default_width(260.0)
            .min_width(180.0)
            .frame(
                egui::Frame::new()
                    .fill(crate::theme::panel_header_bg())
                    .inner_margin(egui::Margin::same(0)),
            )
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(crate::theme::panel_header_bg())
                    .inner_margin(egui::vec2(10.0, 5.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Scene Collection")
                                    .color(crate::theme::text_bright())
                                    .size(11.5)
                                    .strong(),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.menu_button("+", |ui| self.draw_add_entity_menu(ui));
                                },
                            );
                        });
                    });
                ui.painter().hline(
                    ui.available_rect_before_wrap().x_range(),
                    ui.cursor().top(),
                    egui::Stroke::new(1.0_f32, crate::theme::border()),
                );
                ui.add_space(1.0);

                egui::Frame::new()
                    .fill(crate::theme::panel_header_bg())
                    .inner_margin(egui::vec2(4.0, 2.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().id_salt("h_scroll").show(ui, |ui| {
                            if self.scene_entities.is_empty() {
                                ui.add_space(12.0);
                                ui.label(
                                    egui::RichText::new("  (empty)")
                                        .color(crate::theme::text_dim())
                                        .size(11.5),
                                );
                                return;
                            }
                            let p = ui.painter().clone();
                            for &entity in &self.scene_entities.clone() {
                                let name = self.get_entity_name(entity);
                                let selected = self.selected_entity == Some(entity);
                                let icon = icon_for_entity(&name);
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 22.0),
                                    egui::Sense::click(),
                                );
                                if ui.is_rect_visible(rect) {
                                    let bg = if selected {
                                        crate::theme::hover()
                                    } else if resp.hovered() {
                                        egui::Color32::from_rgb(0x30, 0x30, 0x35)
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    };
                                    if bg != egui::Color32::TRANSPARENT {
                                        p.rect_filled(rect, egui::CornerRadius::ZERO, bg);
                                    }
                                    if selected {
                                        let bar = egui::Rect::from_min_size(
                                            rect.left_top(),
                                            egui::vec2(3.0, rect.height()),
                                        );
                                        p.rect_filled(
                                            bar,
                                            egui::CornerRadius::ZERO,
                                            crate::theme::accent_color(),
                                        );
                                    }
                                    draw_icon(&p, rect, icon);
                                    p.text(
                                        rect.left_center() + egui::vec2(24.0, 0.0),
                                        egui::Align2::LEFT_CENTER,
                                        name,
                                        egui::FontId::proportional(12.0),
                                        if selected {
                                            crate::theme::text_bright()
                                        } else {
                                            crate::theme::text_dim()
                                        },
                                    );
                                }
                                if resp.clicked() {
                                    self.selected_entity = Some(entity);
                                }
                            }
                        });
                    });
            });
    }

    fn draw_add_entity_menu(&mut self, ui: &mut egui::Ui) {
        let items: &[(Icon, &str, fn(&mut EditorApp))] = &[
            (Icon::Cube, "Cube", |s| {
                s.add_mesh_entity(
                    "Cube",
                    &schiro_render::Mesh::cube(),
                    glam::Vec3::new(0.0, 0.5, 0.0),
                    None,
                )
            }),
            (Icon::Sphere, "Sphere", |s| {
                let m = render_to_mesh(&schiro_assets::procedural::create_sphere(1.0, 32, 16));
                s.add_mesh_entity(
                    "Sphere",
                    &m,
                    glam::Vec3::new(0.0, 1.5, 0.0),
                    Some(glam::Vec3::new(0.0, 1.5, 0.0)),
                );
            }),
            (Icon::Plane, "Plane", |s| {
                s.add_mesh_entity("Plane", &schiro_render::Mesh::plane(), glam::Vec3::ZERO, None)
            }),
            (Icon::Light, "Directional Light", |s| {
                s.add_empty("Directional Light", glam::Vec3::new(0.0, 3.0, 0.0))
            }),
            (Icon::Empty, "Empty", |s| s.add_empty("Empty", glam::Vec3::ZERO)),
        ];
        for (i, (icon, label, action)) in items.iter().enumerate() {
            if i == 3 {
                ui.separator();
            }
            let (rect, resp) = ui
                .allocate_exact_size(egui::vec2(ui.available_width(), 20.0), egui::Sense::click());
            if ui.is_rect_visible(rect) {
                if resp.hovered() {
                    ui.painter().rect_filled(
                        rect,
                        egui::CornerRadius::same(3),
                        crate::theme::hover(),
                    );
                }
                draw_icon(ui.painter(), rect.shrink(2.0), *icon);
                ui.painter().text(
                    rect.left_center() + egui::vec2(26.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    *label,
                    egui::FontId::proportional(12.0),
                    crate::theme::text_bright(),
                );
            }
            if resp.clicked() {
                action(self);
                ui.close_menu();
            }
        }
    }
}

fn render_to_mesh(asset: &schiro_assets::types::MeshAsset) -> schiro_render::Mesh {
    let mut mesh = schiro_render::Mesh::new(&asset.name);
    for i in 0..asset.positions.len() {
        let t = if i < asset.tangents.len() { asset.tangents[i] } else { [1.0, 0.0, 0.0, 1.0] };
        mesh.vertices.push(schiro_render::mesh::Vertex {
            position: asset.positions[i],
            normal: if i < asset.normals.len() { asset.normals[i] } else { [0.0, 1.0, 0.0] },
            uv: if i < asset.uvs.len() { asset.uvs[i] } else { [0.0, 0.0] },
            tangent: t,
        });
    }
    mesh.indices = asset.indices.clone();
    mesh
}
