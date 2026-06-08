use egui::Key;
use glam::{Mat4, Vec2, Vec3};
use schiro_assets::types::MeshAsset;
use schiro_core::{Aabb, Ray};
use schiro_ecs::World;
use schiro_input::InputSystem;
use schiro_physics::PhysicsWorld;
use schiro_render::Renderer;
use std::sync::Arc;
use tracing::info;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes};

use crate::project::Project;
use crate::viewport::ViewportPanel;

pub struct EditorApp {
    world: World,
    renderer: Option<Renderer>,
    asset_server: schiro_assets::AssetServer,
    input: InputSystem,
    physics: PhysicsWorld,
    project: Project,
    window: Option<Arc<Window>>,
    egui_ctx: egui::Context,
    egui_winit_state: Option<egui_winit::State>,
    viewport_panel: ViewportPanel,
    scene_objects: Vec<SceneObject>,
    selected_index: Option<usize>,
    gizmo_mesh_start: usize,
    gizmo_drag: Option<GizmoDrag>,
    current_tool: EditorTool,
}

struct SceneObject {
    name: String,
    mesh_index: usize,
    transform: Mat4,
    aabb: Aabb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorTool {
    Translate,
    Rotate,
    Scale,
}

#[derive(Debug, Clone, Copy)]
struct GizmoDrag {
    axis: usize,
    obj_index: usize,
}

impl EditorApp {
    pub fn new() -> Self {
        let mut world = World::new();
        schiro_ecs::init(&mut world);
        info!("editor engine initialized");

        let mut app = Self {
            world,
            renderer: None,
            asset_server: schiro_assets::AssetServer::new(),
            input: InputSystem::new(),
            physics: PhysicsWorld::new(),
            project: Project::new("Untitled"),
            window: None,
            egui_ctx: egui::Context::default(),
            egui_winit_state: None,
            viewport_panel: ViewportPanel::new(),
            scene_objects: Vec::new(),
            selected_index: None,
            gizmo_mesh_start: 0,
            gizmo_drag: None,
            current_tool: EditorTool::Translate,
        };

        crate::theme::apply_dark_theme(&app.egui_ctx);
        app
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)?;
        Ok(())
    }

    fn render_frame(&mut self) {
        let raw_input = {
            let egui_state = match self.egui_winit_state.as_mut() {
                Some(s) => s,
                None => return,
            };
            let window = match self.window.as_ref() {
                Some(w) => w,
                None => return,
            };
            egui_state.take_egui_input(window)
        };

        let ctx = self.egui_ctx.clone();
        let full_output = ctx.run(raw_input, |ctx| {
            self.build_editor_ui(ctx);
        });

        if let (Some(state), Some(window)) = (self.egui_winit_state.as_mut(), self.window.as_ref())
        {
            state.handle_platform_output(window, full_output.platform_output.clone());
        }

        let viewport_size =
            (self.viewport_panel.rect.width() as u32, self.viewport_panel.rect.height() as u32);

        let aspect = if viewport_size.1 > 0 {
            viewport_size.0 as f32 / viewport_size.1 as f32
        } else {
            16.0 / 9.0
        };

        self.handle_viewport_input(aspect);

        self.viewport_panel.gizmo_hovered = self.check_gizmo_hover(aspect);

        let renderer = match self.renderer.as_mut() {
            Some(r) => r,
            None => return,
        };

        if viewport_size.0 > 0 && viewport_size.1 > 0 {
            renderer.resize_viewport(viewport_size.0, viewport_size.1);
        }

        update_gizmo_transforms(
            renderer,
            &self.scene_objects,
            self.selected_index,
            self.gizmo_mesh_start,
            self.current_tool,
        );

        let camera_uniform = self.viewport_panel.camera.to_uniform(aspect);

        if let Err(wgpu::SurfaceError::OutOfMemory) =
            renderer.render(&ctx, full_output, &camera_uniform)
        {
            panic!("GPU out of memory");
        }
    }

    fn handle_viewport_input(&mut self, aspect: f32) {
        let vp_size =
            egui::vec2(self.viewport_panel.rect.width(), self.viewport_panel.rect.height());

        for key in self.viewport_panel.keys_pressed.clone() {
            match key {
                Key::W => self.current_tool = EditorTool::Translate,
                Key::E => self.current_tool = EditorTool::Rotate,
                Key::R => self.current_tool = EditorTool::Scale,
                _ => {}
            }
        }

        let clicked = self.viewport_panel.clicked_pos.take();

        if self.viewport_panel.gizmo_held {
            if self.gizmo_drag.is_none() {
                if let Some(sel) = self.selected_index {
                    if let Some((mx, my)) = self.viewport_panel.gizmo_press_pos {
                        let ray = self.viewport_panel.camera.screen_to_ray(
                            Vec2::new(mx, my),
                            Vec2::new(vp_size.x, vp_size.y),
                            aspect,
                        );
                        let pos = self.scene_objects[sel].transform.w_axis.truncate();
                        let (axis, _) = self.pick_gizmo_axis(&ray, pos, self.current_tool);
                        if axis < 3 {
                            self.gizmo_drag = Some(GizmoDrag { axis, obj_index: sel });
                        }
                    }
                }
            }
        } else {
            self.gizmo_drag = None;
        }

        if let Some((x, y)) = clicked {
            let ray = self.viewport_panel.camera.screen_to_ray(
                Vec2::new(x, y),
                Vec2::new(vp_size.x, vp_size.y),
                aspect,
            );

            let mut best: Option<(usize, f32)> = None;
            for (i, obj) in self.scene_objects.iter().enumerate() {
                if let Some(t) = ray.intersects_aabb(&obj.aabb) {
                    if best.map_or(true, |(_, d)| t < d) {
                        best = Some((i, t));
                    }
                }
            }
            self.selected_index = best.map(|(i, _)| i);
        }

        if let Some(drag) = self.gizmo_drag {
            let (dx, dy) = self.viewport_panel.mouse_delta;
            if dx != 0.0 || dy != 0.0 {
                match self.current_tool {
                    EditorTool::Translate => self.drag_translate(drag, dx, dy),
                    EditorTool::Rotate => self.drag_rotate(drag, dx),
                    EditorTool::Scale => self.drag_scale(drag, dx),
                }
            }
        }
    }

    fn drag_translate(&mut self, drag: GizmoDrag, dx: f32, dy: f32) {
        let cam = &self.viewport_panel.camera;
        let right = cam.view_matrix().inverse().x_axis.truncate();
        let up = cam.view_matrix().inverse().y_axis.truncate();
        let sensitivity = cam.distance * 0.002;
        let world_delta = right * dx * sensitivity + up * (-dy) * sensitivity;

        let axis_dir = match drag.axis {
            0 => Vec3::X,
            1 => Vec3::Y,
            2 => Vec3::Z,
            _ => return,
        };
        let move_amount = world_delta.dot(axis_dir);
        self.scene_objects[drag.obj_index].transform.w_axis += axis_dir.extend(0.0) * move_amount;
        self.apply_object_transform(drag.obj_index);
    }

    fn drag_rotate(&mut self, drag: GizmoDrag, dx: f32) {
        let angle = dx * 0.01;
        let axis = match drag.axis {
            0 => Vec3::X,
            1 => Vec3::Y,
            2 => Vec3::Z,
            _ => return,
        };
        let rot = glam::Quat::from_axis_angle(axis, angle);
        let t = &mut self.scene_objects[drag.obj_index].transform;
        let pos = t.w_axis.truncate();
        let current_rot =
            glam::Quat::from_mat4(&Mat4::from_cols(t.x_axis, t.y_axis, t.z_axis, glam::Vec4::W));
        *t = Mat4::from_rotation_translation(rot * current_rot, pos);
        self.apply_object_transform(drag.obj_index);
    }

    fn drag_scale(&mut self, drag: GizmoDrag, dx: f32) {
        let scale_factor = 1.0 + dx * 0.01;
        let axis = match drag.axis {
            0 => Vec3::X,
            1 => Vec3::Y,
            2 => Vec3::Z,
            _ => return,
        };
        let t = &mut self.scene_objects[drag.obj_index].transform;
        let pos = t.w_axis.truncate();
        let current_rot =
            glam::Quat::from_mat4(&Mat4::from_cols(t.x_axis, t.y_axis, t.z_axis, glam::Vec4::W));
        let mut scale = Vec3::ONE;
        if axis == Vec3::X {
            scale.x = scale_factor;
        }
        if axis == Vec3::Y {
            scale.y = scale_factor;
        }
        if axis == Vec3::Z {
            scale.z = scale_factor;
        }
        *t = Mat4::from_scale_rotation_translation(scale, current_rot, pos);
        self.apply_object_transform(drag.obj_index);
    }

    fn apply_object_transform(&self, obj_index: usize) {
        if let Some(ref renderer) = self.renderer {
            let idx = self.scene_objects[obj_index].mesh_index;
            renderer.update_mesh_transform(idx, &self.scene_objects[obj_index].transform);
        }
    }

    fn check_gizmo_hover(&self, aspect: f32) -> bool {
        if self.selected_index.is_none() {
            return false;
        }
        let sel = self.selected_index.unwrap();
        let pos = self.scene_objects[sel].transform.w_axis.truncate();

        let vp_size =
            egui::vec2(self.viewport_panel.rect.width(), self.viewport_panel.rect.height());
        if vp_size.x <= 0.0 || vp_size.y <= 0.0 {
            return false;
        }

        let (mx, my) = (self.viewport_panel.prev_mouse.0, self.viewport_panel.prev_mouse.1);
        if mx == 0.0 && my == 0.0 {
            return false;
        }

        let ray = self.viewport_panel.camera.screen_to_ray(
            Vec2::new(mx, my),
            Vec2::new(vp_size.x, vp_size.y),
            aspect,
        );
        let (axis, _) = self.pick_gizmo_axis(&ray, pos, self.current_tool);
        axis < 3
    }

    fn pick_gizmo_axis(&self, ray: &Ray, center: Vec3, tool: EditorTool) -> (usize, f32) {
        match tool {
            EditorTool::Translate => {
                let shaft_half = 0.04;
                let tip_half = 0.08;
                let shaft_len = 0.8;
                for (i, dir) in [Vec3::X, Vec3::Y, Vec3::Z].iter().enumerate() {
                    let sa = aabb_along_axis(center, *dir, 0.0, shaft_len, shaft_half);
                    if let Some(t) = ray.intersects_aabb(&sa) {
                        return (i, t);
                    }
                    let ta = aabb_along_axis(center, *dir, shaft_len, 0.2, tip_half);
                    if let Some(t) = ray.intersects_aabb(&ta) {
                        return (i, t);
                    }
                }
            }
            EditorTool::Rotate => {
                let ring_radius = 1.0;
                let ring_thick = 0.05;
                for (i, axis) in [Vec3::X, Vec3::Y, Vec3::Z].iter().enumerate() {
                    let aabb = ring_aabb(center, *axis, ring_radius, ring_thick);
                    if let Some(t) = ray.intersects_aabb(&aabb) {
                        return (i, t);
                    }
                }
            }
            EditorTool::Scale => {
                let handle_half = 0.07;
                let distance = 0.9;
                for (i, dir) in [Vec3::X, Vec3::Y, Vec3::Z].iter().enumerate() {
                    let aabb = aabb_along_axis(
                        center,
                        *dir,
                        distance - handle_half,
                        handle_half * 2.0,
                        handle_half,
                    );
                    if let Some(t) = ray.intersects_aabb(&aabb) {
                        return (i, t);
                    }
                }
            }
        }
        (3, f32::MAX)
    }
}

fn update_gizmo_transforms(
    renderer: &mut Renderer,
    scene_objects: &[SceneObject],
    selected_index: Option<usize>,
    gizmo_mesh_start: usize,
    tool: EditorTool,
) {
    let hide = Mat4::from_scale(Vec3::ZERO);

    if let Some(sel) = selected_index {
        let pos = scene_objects[sel].transform.w_axis.truncate();
        let transform = Mat4::from_translation(pos);

        let (translate_range, rotate_range, scale_range) = match tool {
            EditorTool::Translate => (0..6, 6..6, 9..9),
            EditorTool::Rotate => (0..0, 6..9, 9..9),
            EditorTool::Scale => (0..0, 6..6, 9..12),
        };

        for i in 0..12 {
            let idx = gizmo_mesh_start + i;
            let show = (i as i32 >= translate_range.start && (i as i32) < translate_range.end)
                || (i as i32 >= rotate_range.start && (i as i32) < rotate_range.end)
                || (i as i32 >= scale_range.start && (i as i32) < scale_range.end);
            renderer.update_mesh_transform(idx, if show { &transform } else { &hide });
        }
    } else {
        for i in 0..12 {
            renderer.update_mesh_transform(gizmo_mesh_start + i, &hide);
        }
    }
}

impl EditorApp {
    fn build_editor_ui(&mut self, ctx: &egui::Context) {
        self.build_menu_bar(ctx);
        self.build_hierarchy_panel(ctx);
        self.build_inspector_panel(ctx);
        self.build_status_bar(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0))
            .show(ctx, |ui| {
                let viewport_frame = egui::Frame::none()
                    .fill(crate::theme::faint_bg_color())
                    .stroke(egui::Stroke::new(1.0, crate::theme::faint_bg_color()))
                    .inner_margin(egui::Margin::same(0));

                viewport_frame.show(ui, |ui| {
                    let tex_id = self
                        .renderer
                        .as_ref()
                        .and_then(|r| r.viewport.as_ref())
                        .and_then(|vp| vp.egui_texture_id);

                    let vp_size = self
                        .renderer
                        .as_ref()
                        .and_then(|r| r.viewport.as_ref())
                        .map(|vp| vp.size)
                        .unwrap_or((1280, 720));

                    if tex_id.is_some() {
                        self.viewport_panel.show(ui, tex_id, vp_size);
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                egui::RichText::new("Viewport")
                                    .color(crate::theme::text_dim())
                                    .size(16.0),
                            );
                        });
                    }
                });
            });
    }

    fn build_menu_bar(&mut self, _ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar")
            .frame(
                egui::Frame::none()
                    .fill(crate::theme::panel_header_bg())
                    .inner_margin(egui::vec2(8.0, 2.0)),
            )
            .show(_ctx, |ui| {
                ui.horizontal(|ui| {
                    egui::menu::bar(ui, |ui| {
                        ui.style_mut().visuals.widgets.inactive.corner_radius =
                            egui::CornerRadius::same(4);
                        ui.style_mut().visuals.widgets.hovered.corner_radius =
                            egui::CornerRadius::same(4);
                        ui.style_mut().visuals.widgets.active.corner_radius =
                            egui::CornerRadius::same(4);

                        ui.menu_button("File", |ui| {
                            if ui.button("New Project").clicked() {}
                            if ui.button("Open Project").clicked() {}
                            ui.separator();
                            if ui.button("Exit").clicked() {
                                std::process::exit(0);
                            }
                        });
                        ui.menu_button("Edit", |ui| {
                            if ui.button("Undo").clicked() {}
                            if ui.button("Redo").clicked() {}
                        });
                        ui.menu_button("View", |ui| {
                            if ui.button("Asset Browser").clicked() {}
                            if ui.button("Hierarchy").clicked() {}
                        });
                        ui.menu_button("Help", |ui| if ui.button("About").clicked() {});
                    });

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(4.0);

                    self.draw_tool_button(ui, "\u{2194}", "Translate", EditorTool::Translate);
                    self.draw_tool_button(ui, "\u{21BB}", "Rotate", EditorTool::Rotate);
                    self.draw_tool_button(ui, "\u{25A1}", "Scale", EditorTool::Scale);
                });
            });
    }

    fn draw_tool_button(&mut self, ui: &mut egui::Ui, icon: &str, _label: &str, tool: EditorTool) {
        let selected = self.current_tool == tool;
        let mut visuals = ui.style().visuals.widgets.inactive.clone();
        if selected {
            visuals.bg_fill = crate::theme::accent_color();
            visuals.fg_stroke = egui::Stroke::new(1.5, crate::theme::text_bright());
        }
        visuals.corner_radius = egui::CornerRadius::same(4);

        let (rect, response) = ui.allocate_exact_size(egui::vec2(32.0, 22.0), egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let fill = if response.hovered() && !selected {
                ui.style().visuals.widgets.hovered.bg_fill
            } else {
                visuals.bg_fill
            };
            ui.painter().rect(
                rect,
                egui::CornerRadius::same(4),
                fill,
                if selected {
                    egui::Stroke::NONE
                } else {
                    ui.style().visuals.widgets.inactive.bg_stroke
                },
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(14.0),
                if selected { crate::theme::text_bright() } else { crate::theme::text_dim() },
            );
        }

        if response.clicked() {
            self.current_tool = tool;
        }

        response.on_hover_text(format!(
            "{} [{}]",
            _label,
            match tool {
                EditorTool::Translate => "W",
                EditorTool::Rotate => "E",
                EditorTool::Scale => "R",
            }
        ));
    }

    fn build_hierarchy_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("hierarchy_panel")
            .resizable(true)
            .default_width(260.0)
            .min_width(180.0)
            .frame(
                egui::Frame::none()
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
                });
                ui.add_space(6.0);

                egui::ScrollArea::vertical().id_salt("hierarchy_scroll").show(ui, |ui| {
                    if self.scene_objects.is_empty() {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("  (empty scene)")
                                .color(crate::theme::text_dim())
                                .size(12.0),
                        );
                        return;
                    }

                    for (i, obj) in self.scene_objects.iter().enumerate() {
                        let is_selected = self.selected_index == Some(i);
                        let response = self.draw_hierarchy_item(ui, obj, is_selected, i == 0);
                        if response.clicked() {
                            self.selected_index = Some(i);
                        }
                    }
                });
            });
    }

    fn draw_hierarchy_item(
        &self,
        ui: &mut egui::Ui,
        obj: &SceneObject,
        selected: bool,
        _first: bool,
    ) -> egui::Response {
        let icon = if obj.name.contains("Sphere") {
            "\u{25C9}"
        } else if obj.name.contains("Grid") {
            "\u{25A6}"
        } else {
            "\u{25A0}"
        };

        let label_text = format!("{}  {}", icon, obj.name);

        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 24.0), egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = if selected {
                let mut v = ui.style().visuals.widgets.active.clone();
                v.bg_fill = crate::theme::faint_bg_color();
                v.bg_stroke = egui::Stroke::new(1.0, crate::theme::accent_color());
                v
            } else if response.hovered() {
                ui.style().visuals.widgets.hovered.clone()
            } else {
                ui.style().visuals.widgets.inactive.clone()
            };

            let corner = egui::CornerRadius::same(4);

            ui.painter().rect(
                rect.shrink(1.0),
                corner,
                visuals.bg_fill,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            );

            let text_pos = rect.left_center() + egui::vec2(12.0, 0.0);
            ui.painter().text(
                text_pos,
                egui::Align2::LEFT_CENTER,
                &label_text,
                egui::FontId::monospace(12.5),
                visuals.fg_stroke.color,
            );
        }

        response
    }

    fn build_inspector_panel(&mut self, ctx: &egui::Context) {
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
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Inspector")
                            .color(crate::theme::text_bright())
                            .size(13.0)
                            .strong(),
                    );
                });
                ui.add_space(6.0);

                if let Some(idx) = self.selected_index {
                    let obj = &self.scene_objects[idx];
                    egui::ScrollArea::vertical().id_salt("inspector_scroll").show(ui, |ui| {
                        self.draw_inspector_header(ui, obj);
                        self.draw_inspector_transform(ui, obj);
                        self.draw_inspector_bounds(ui, obj);
                    });
                } else {
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("No object selected")
                            .color(crate::theme::text_dim())
                            .size(12.0),
                    );
                    ui.label(
                        egui::RichText::new(
                            "Click an object in the viewport\nor the Hierarchy panel.",
                        )
                        .color(crate::theme::text_dim())
                        .size(11.0),
                    );
                }
            });
    }

    fn draw_inspector_header(&self, ui: &mut egui::Ui, obj: &SceneObject) {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&obj.name)
                    .color(crate::theme::text_bright())
                    .size(14.0)
                    .strong(),
            );
        });
        ui.add_space(6.0);
    }

    fn draw_inspector_transform(&self, ui: &mut egui::Ui, obj: &SceneObject) {
        let bg = crate::theme::faint_bg_color();
        let corner = egui::CornerRadius::same(4);

        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 80.0), egui::Sense::hover());
        ui.painter().rect(rect, corner, bg, egui::Stroke::NONE, egui::StrokeKind::Inside);

        let pos = obj.transform.w_axis.truncate();
        let content_rect = rect.shrink(8.0);

        let mut y = content_rect.min.y + 4.0;
        ui.painter().text(
            egui::pos2(content_rect.min.x, y),
            egui::Align2::LEFT_TOP,
            "Transform",
            egui::FontId::proportional(11.0),
            crate::theme::text_dim(),
        );
        y += 16.0;

        let fields = [("X", pos.x), ("Y", pos.y), ("Z", pos.z)];
        for (label, value) in &fields {
            ui.painter().text(
                egui::pos2(content_rect.min.x, y),
                egui::Align2::LEFT_TOP,
                *label,
                egui::FontId::proportional(11.5),
                crate::theme::accent_color(),
            );
            ui.painter().text(
                egui::pos2(content_rect.min.x + 18.0, y),
                egui::Align2::LEFT_TOP,
                format!("{:.3}", value),
                egui::FontId::monospace(12.0),
                crate::theme::text_bright(),
            );
            y += 17.0;
        }
        ui.add_space(6.0);
    }

    fn draw_inspector_bounds(&self, ui: &mut egui::Ui, obj: &SceneObject) {
        let bg = crate::theme::faint_bg_color();
        let corner = egui::CornerRadius::same(4);

        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 60.0), egui::Sense::hover());
        ui.painter().rect(rect, corner, bg, egui::Stroke::NONE, egui::StrokeKind::Inside);

        let content_rect = rect.shrink(8.0);
        let mut y = content_rect.min.y + 4.0;
        ui.painter().text(
            egui::pos2(content_rect.min.x, y),
            egui::Align2::LEFT_TOP,
            "Bounding Box",
            egui::FontId::proportional(11.0),
            crate::theme::text_dim(),
        );
        y += 16.0;

        ui.painter().text(
            egui::pos2(content_rect.min.x, y),
            egui::Align2::LEFT_TOP,
            format!("Min: ({:.2}, {:.2}, {:.2})", obj.aabb.min.x, obj.aabb.min.y, obj.aabb.min.z),
            egui::FontId::monospace(11.5),
            crate::theme::text_bright(),
        );
        y += 14.0;
        ui.painter().text(
            egui::pos2(content_rect.min.x, y),
            egui::Align2::LEFT_TOP,
            format!("Max: ({:.2}, {:.2}, {:.2})", obj.aabb.max.x, obj.aabb.max.y, obj.aabb.max.z),
            egui::FontId::monospace(11.5),
            crate::theme::text_bright(),
        );
        ui.add_space(4.0);
    }

    fn build_status_bar(&mut self, _ctx: &egui::Context) {
        let tool_name = match self.current_tool {
            EditorTool::Translate => "Translate [W]",
            EditorTool::Rotate => "Rotate [E]",
            EditorTool::Scale => "Scale [R]",
        };

        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::none()
                    .fill(crate::theme::panel_header_bg())
                    .inner_margin(egui::vec2(12.0, 2.0)),
            )
            .show(_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("SchiroEngine v0.1  |  {}", tool_name))
                            .color(crate::theme::text_dim())
                            .size(11.0),
                    );
                });
            });
    }
}

impl ApplicationHandler for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_title("SchiroEngine Editor")
            .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080))
            .with_maximized(true);

        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        let window_size = window.inner_size();
        let mut renderer = pollster::block_on(Renderer::new(
            Arc::clone(&window),
            (window_size.width, window_size.height),
        ))
        .expect("failed to create renderer");

        init_scene(
            &mut renderer,
            &self.asset_server,
            &mut self.scene_objects,
            &mut self.gizmo_mesh_start,
        );

        let egui_winit_state = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::default(),
            &window,
            None,
            None,
            None,
        );

        self.renderer = Some(renderer);
        self.egui_winit_state = Some(egui_winit_state);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        self.input.process_window_event(&event);

        if let Some(ref mut state) = self.egui_winit_state {
            if let Some(ref window) = self.window {
                let _ = state.on_window_event(window, &event);
            }
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.render_frame(),
            WindowEvent::Resized(physical_size) => {
                if let Some(ref mut renderer) = self.renderer {
                    renderer.resize(physical_size.width, physical_size.height);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}

fn init_scene(
    renderer: &mut Renderer,
    asset_server: &schiro_assets::AssetServer,
    scene: &mut Vec<SceneObject>,
    gizmo_mesh_start: &mut usize,
) {
    let gizmo = schiro_render::GizmoMeshes::new();
    *gizmo_mesh_start = renderer.mesh_count();
    let hide = Mat4::from_scale(Vec3::ZERO);
    for part in [
        &gizmo.x_shaft,
        &gizmo.x_tip,
        &gizmo.y_shaft,
        &gizmo.y_tip,
        &gizmo.z_shaft,
        &gizmo.z_tip,
        &gizmo.rot_x,
        &gizmo.rot_y,
        &gizmo.rot_z,
        &gizmo.scale_x,
        &gizmo.scale_y,
        &gizmo.scale_z,
    ] {
        renderer.add_mesh(part, &hide);
    }

    let sphere_asset = schiro_assets::procedural::create_sphere(1.0, 32, 16);
    let sphere = asset_server.load("procedural://sphere", |_| Ok(sphere_asset.clone())).unwrap();

    let sphere_mesh = asset_to_render_mesh(&sphere);
    let transform = Mat4::from_translation(Vec3::new(0.0, 1.5, 0.0));
    let aabb = Aabb::from_points(&sphere.positions).transform(&transform);

    let mesh_index = renderer.mesh_count();
    renderer.add_mesh(&sphere_mesh, &transform);
    scene.push(SceneObject { name: "Sphere".into(), mesh_index, transform, aabb });

    let grid_mesh = schiro_render::Mesh::grid(10, 10, 1.0);
    let grid_transform = Mat4::IDENTITY;
    let grid_aabb = Aabb::new(Vec3::new(-5.0, -0.01, -5.0), Vec3::new(5.0, 0.01, 5.0));

    let grid_index = renderer.mesh_count();
    renderer.add_mesh(&grid_mesh, &grid_transform);
    scene.push(SceneObject {
        name: "Grid".into(),
        mesh_index: grid_index,
        transform: grid_transform,
        aabb: grid_aabb,
    });

    info!("scene initialized: {} objects", scene.len());
}

fn asset_to_render_mesh(asset: &MeshAsset) -> schiro_render::Mesh {
    let mut mesh = schiro_render::Mesh::new(&asset.name);
    for i in 0..asset.positions.len() {
        let tangent =
            if i < asset.tangents.len() { asset.tangents[i] } else { [1.0, 0.0, 0.0, 1.0] };
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

fn aabb_along_axis(center: Vec3, dir: Vec3, start: f32, length: f32, half_width: f32) -> Aabb {
    let axis_end = center + dir * (start + length);
    let axis_start = center + dir * start;
    let half = Vec3::splat(half_width);

    let mut min = axis_start.min(axis_end) - half;
    let mut max = axis_start.max(axis_end) + half;

    min = min.min(center - half);
    max = max.max(center + half);

    Aabb::new(min, max)
}

fn ring_aabb(center: Vec3, axis: Vec3, radius: f32, thickness: f32) -> Aabb {
    let half = Vec3::splat(radius + thickness);
    let mut min = center - half;
    let mut max = center + half;
    if axis == Vec3::X {
        min.x = center.x - thickness;
        max.x = center.x + thickness;
    }
    if axis == Vec3::Y {
        min.y = center.y - thickness;
        max.y = center.y + thickness;
    }
    if axis == Vec3::Z {
        min.z = center.z - thickness;
        max.z = center.z + thickness;
    }
    Aabb::new(min, max)
}
