//! Hierarchy panel: lists every scene entity and lets the user select
//! one by clicking on it.

use crate::app::EditorApp;

impl EditorApp {
    /// Builds the left-hand hierarchy panel.
    pub fn build_hierarchy_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("hierarchy_panel")
            .resizable(true)
            .default_width(260.0)
            .min_width(180.0)
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::vec2(8.0, 6.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Scene Hierarchy")
                            .color(crate::theme::text_bright())
                            .size(13.0)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.menu_button("+ \u{25BC}", |ui| self.draw_add_entity_menu(ui));
                    });
                });
                ui.add_space(4.0);

                let response = egui::ScrollArea::vertical()
                    .id_salt("h_scroll")
                    .show(ui, |ui| {
                        if self.scene_entities.is_empty() {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("  (empty scene)")
                                    .color(crate::theme::text_dim())
                                    .size(12.0),
                            );
                            return;
                        }
                        for &entity in &self.scene_entities.iter().copied().collect::<Vec<_>>() {
                            let name = self.get_entity_name(entity);
                            let selected = self.selected_entity == Some(entity);
                            let icon = if name.contains("Sphere") || name.contains("Cube") || name.contains("Plane") {
                                "\u{25A0}"
                            } else if name.contains("Light") {
                                "\u{2606}"
                            } else if name.contains("Grid") {
                                "\u{25A6}"
                            } else {
                                "\u{25CB}"
                            };
                            let (rect, resp) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), 24.0),
                                egui::Sense::click(),
                            );
                            if ui.is_rect_visible(rect) {
                                let vis = if selected {
                                    let mut v = ui.style().visuals.widgets.active.clone();
                                    v.bg_fill = crate::theme::faint_bg_color();
                                    v.bg_stroke =
                                        egui::Stroke::new(1.0_f32, crate::theme::accent_color());
                                    v
                                } else if resp.hovered() {
                                    ui.style().visuals.widgets.hovered.clone()
                                } else {
                                    ui.style().visuals.widgets.inactive.clone()
                                };
                                ui.painter().rect(
                                    rect.shrink(1.0),
                                    egui::CornerRadius::same(4),
                                    vis.bg_fill,
                                    vis.bg_stroke,
                                    egui::StrokeKind::Inside,
                                );
                                ui.painter().text(
                                    rect.left_center() + egui::vec2(12.0, 0.0),
                                    egui::Align2::LEFT_CENTER,
                                    format!("{}  {}", icon, name),
                                    egui::FontId::monospace(12.5),
                                    vis.fg_stroke.color,
                                );
                            }
                            if resp.clicked() {
                                self.selected_entity = Some(entity);
                            }
                        }
                    });
                let _ = response;
            });
    }

    /// Menu drawn inside the "+" button and on right-click.
    fn draw_add_entity_menu(&mut self, ui: &mut egui::Ui) {
        if ui.button("Cube").clicked() {
            self.add_mesh_entity("Cube", &schiro_render::Mesh::cube(), glam::Vec3::new(0.0, 0.5, 0.0), None);
            ui.close_menu();
        }
        if ui.button("Sphere").clicked() {
            let mesh = render_to_mesh(&schiro_assets::procedural::create_sphere(1.0, 32, 16));
            self.add_mesh_entity("Sphere", &mesh, glam::Vec3::new(0.0, 1.5, 0.0), Some(glam::Vec3::new(0.0, 1.5, 0.0)));
            ui.close_menu();
        }
        if ui.button("Plane").clicked() {
            self.add_mesh_entity("Plane", &schiro_render::Mesh::plane(), glam::Vec3::ZERO, None);
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Directional Light").clicked() {
            self.add_empty("Directional Light", glam::Vec3::new(0.0, 3.0, 0.0));
            ui.close_menu();
        }
        if ui.button("Empty").clicked() {
            self.add_empty("Empty", glam::Vec3::ZERO);
            ui.close_menu();
        }
    }
}

fn render_to_mesh(asset: &schiro_assets::types::MeshAsset) -> schiro_render::Mesh {
    let mut mesh = schiro_render::Mesh::new(&asset.name);
    for i in 0..asset.positions.len() {
        let tangent = if i < asset.tangents.len() { asset.tangents[i] } else { [1.0, 0.0, 0.0, 1.0] };
        mesh.vertices.push(schiro_render::mesh::Vertex {
            position: asset.positions[i],
            normal: if i < asset.normals.len() { asset.normals[i] } else { [0.0, 1.0, 0.0] },
            uv: if i < asset.uvs.len() { asset.uvs[i] } else { [0.0, 0.0] },
            tangent,
        });
    }
    mesh.indices = asset.indices.clone();
    mesh
}
