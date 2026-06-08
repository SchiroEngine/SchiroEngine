use std::collections::HashMap;
use std::sync::Arc;

use bevy_ecs::prelude::*;
use egui::Key;
use glam::{Mat4, Vec2, Vec3};
use schiro_assets::types::MeshAsset;
use schiro_core::{Aabb, Ray};
use schiro_ecs::{
    components::{MeshRenderer, Rotator, Transform},
    systems::Time,
    World,
};
use schiro_input::InputSystem;
use schiro_physics::PhysicsWorld;
use schiro_render::Renderer;
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
    scene_entities: Vec<Entity>,
    selected_entity: Option<Entity>,
    entity_mesh_map: HashMap<Entity, usize>,
    gizmo_mesh_start: usize,
    gizmo_drag: Option<GizmoDrag>,
    current_tool: EditorTool,
    playing: bool,
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
    entity: Entity,
}

impl EditorApp {
    pub fn new() -> Self {
        let mut world = World::new();
        schiro_ecs::init(&mut world);
        info!("editor engine initialized");

        let app = Self {
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
            scene_entities: Vec::new(),
            selected_entity: None,
            entity_mesh_map: HashMap::new(),
            gizmo_mesh_start: 0,
            gizmo_drag: None,
            current_tool: EditorTool::Translate,
            playing: false,
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
        if self.playing {
            self.run_game_systems();
        }

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

        let renderer = match self.renderer.as_mut() {
            Some(r) => r,
            None => return,
        };

        if viewport_size.0 > 0 && viewport_size.1 > 0 {
            renderer.resize_viewport(viewport_size.0, viewport_size.1);
        }
        {
            let mut query = self.world.query::<(Entity, &Transform, &MeshRenderer)>();
            for (entity, transform, _) in query.iter(&self.world) {
                if let Some(&mesh_idx) = self.entity_mesh_map.get(&entity) {
                    let mat = transform.compute_matrix();
                    renderer.update_mesh_transform(mesh_idx, &mat);
                }
            }
        }

        update_gizmo_transforms(
            renderer,
            &self.world,
            self.selected_entity,
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

    fn run_game_systems(&mut self) {
        let mut time = self.world.resource_mut::<Time>();
        time.update(0.016);

        let mut schedule = Schedule::default();
        schedule.add_systems(schiro_ecs::systems::rotate_entities);
        schedule.add_systems(schiro_ecs::systems::propagate_transforms);
        schedule.run(&mut self.world);
    }

    fn sync_ecs_to_renderer(
        renderer: &mut Renderer,
        world: &mut World,
        mesh_map: &HashMap<Entity, usize>,
    ) {
        let mut query = world.query::<(Entity, &Transform, &MeshRenderer)>();

        for (entity, transform, _) in query.iter(world) {
            if let Some(&mesh_idx) = mesh_map.get(&entity) {
                let mat = transform.compute_matrix();
                renderer.update_mesh_transform(mesh_idx, &mat);
            }
        }
    }

    fn get_entity_name(&self, entity: Entity) -> String {
        self.world
            .get::<schiro_ecs::components::Name>(entity)
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("Entity {}", entity.index()))
    }

    fn get_entity_transform(&self, entity: Entity) -> Transform {
        self.world.get::<Transform>(entity).copied().unwrap_or_default()
    }

    fn get_entity_aabb(&self, entity: Entity) -> Option<Aabb> {
        let transform = self.get_entity_transform(entity);
        let mesh = self.world.get::<MeshRenderer>(entity)?;
        let mesh_idx = self.entity_mesh_map.get(&entity)?;
        // For simplicity, estimate AABB from transform scale
        let s = transform.scale;
        let min = transform.translation - s;
        let max = transform.translation + s;
        Some(Aabb::new(min, max))
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

        self.viewport_panel.gizmo_hovered = self.check_gizmo_hover(aspect);

        let clicked = self.viewport_panel.clicked_pos.take();

        if self.viewport_panel.gizmo_held && self.gizmo_drag.is_none() {
            if let Some(entity) = self.selected_entity {
                if let Some((mx, my)) = self.viewport_panel.gizmo_press_pos {
                    let ray = self.viewport_panel.camera.screen_to_ray(
                        Vec2::new(mx, my),
                        Vec2::new(vp_size.x, vp_size.y),
                        aspect,
                    );
                    let pos = self.get_entity_transform(entity).translation;
                    let (axis, _) = self.pick_gizmo_axis(&ray, pos, self.current_tool);
                    if axis < 3 {
                        self.gizmo_drag = Some(GizmoDrag { axis, entity });
                    }
                }
            }
        }

        if !self.viewport_panel.gizmo_held {
            self.gizmo_drag = None;
        }

        if let Some((x, y)) = clicked {
            let ray = self.viewport_panel.camera.screen_to_ray(
                Vec2::new(x, y),
                Vec2::new(vp_size.x, vp_size.y),
                aspect,
            );

            let mut best: Option<(Entity, f32)> = None;
            for &entity in &self.scene_entities {
                let pos = self.get_entity_transform(entity).translation;
                let aabb = Aabb::new(pos - Vec3::splat(0.5), pos + Vec3::splat(0.5));
                if let Some(t) = ray.intersects_aabb(&aabb) {
                    if best.map_or(true, |(_, d)| t < d) {
                        best = Some((entity, t));
                    }
                }
            }
            self.selected_entity = best.map(|(e, _)| e);
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

    fn check_gizmo_hover(&self, aspect: f32) -> bool {
        let Some(entity) = self.selected_entity else { return false };
        let pos = self.get_entity_transform(entity).translation;
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
                let hw = 0.04;
                let th = 0.08;
                for (i, dir) in [Vec3::X, Vec3::Y, Vec3::Z].iter().enumerate() {
                    if let Some(t) =
                        ray.intersects_aabb(&aabb_along_axis(center, *dir, 0.0, 0.8, hw))
                    {
                        return (i, t);
                    }
                    if let Some(t) =
                        ray.intersects_aabb(&aabb_along_axis(center, *dir, 0.8, 0.2, th))
                    {
                        return (i, t);
                    }
                }
            }
            EditorTool::Rotate => {
                for (i, axis) in [Vec3::X, Vec3::Y, Vec3::Z].iter().enumerate() {
                    if let Some(t) = ray.intersects_aabb(&ring_aabb(center, *axis, 1.0, 0.05)) {
                        return (i, t);
                    }
                }
            }
            EditorTool::Scale => {
                for (i, dir) in [Vec3::X, Vec3::Y, Vec3::Z].iter().enumerate() {
                    if let Some(t) =
                        ray.intersects_aabb(&aabb_along_axis(center, *dir, 0.83, 0.14, 0.07))
                    {
                        return (i, t);
                    }
                }
            }
        }
        (3, f32::MAX)
    }

    fn drag_translate(&mut self, drag: GizmoDrag, dx: f32, dy: f32) {
        let cam = &self.viewport_panel.camera;
        let right = cam.view_matrix().inverse().x_axis.truncate();
        let up = cam.view_matrix().inverse().y_axis.truncate();
        let world_delta = right * dx * cam.distance * 0.002 + up * (-dy) * cam.distance * 0.002;
        let axis_dir = match drag.axis {
            0 => Vec3::X,
            1 => Vec3::Y,
            2 => Vec3::Z,
            _ => return,
        };
        let amount = world_delta.dot(axis_dir);

        if let Some(mut t) = self.world.get_mut::<Transform>(drag.entity) {
            t.translation += axis_dir * amount;
        }
    }

    fn drag_rotate(&mut self, drag: GizmoDrag, dx: f32) {
        let axis = match drag.axis {
            0 => Vec3::X,
            1 => Vec3::Y,
            2 => Vec3::Z,
            _ => return,
        };
        let rot = glam::Quat::from_axis_angle(axis, dx * 0.01);
        if let Some(mut t) = self.world.get_mut::<Transform>(drag.entity) {
            t.rotation = rot * t.rotation;
        }
    }

    fn drag_scale(&mut self, drag: GizmoDrag, dx: f32) {
        let factor = 1.0 + dx * 0.01;
        if let Some(mut t) = self.world.get_mut::<Transform>(drag.entity) {
            match drag.axis {
                0 => t.scale.x *= factor,
                1 => t.scale.y *= factor,
                2 => t.scale.z *= factor,
                _ => {}
            }
        }
    }
}

fn aabb_along_axis(center: Vec3, dir: Vec3, start: f32, length: f32, hw: f32) -> Aabb {
    let a = center + dir * start;
    let b = center + dir * (start + length);
    Aabb::new(a.min(b) - Vec3::splat(hw), a.max(b) + Vec3::splat(hw))
}

fn ring_aabb(center: Vec3, axis: Vec3, radius: f32, thickness: f32) -> Aabb {
    let h = Vec3::splat(radius + thickness);
    let mut min = center - h;
    let mut max = center + h;
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

impl EditorApp {
    fn build_menu_bar(&mut self, _ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar")
            .frame(
                egui::Frame::none()
                    .fill(crate::theme::panel_header_bg())
                    .inner_margin(egui::vec2(8.0, 1.0)),
            )
            .show(_ctx, |ui| {
                egui::menu::bar(ui, |ui| {
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
                });
            });

        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::none()
                    .fill(crate::theme::panel_header_bg())
                    .inner_margin(egui::vec2(8.0, 1.0)),
            )
            .show_separator_line(false)
            .show(_ctx, |ui| {
                ui.horizontal(|ui| {
                    self.draw_tool_button(ui, "\u{2194}", "Translate", EditorTool::Translate);
                    self.draw_tool_button(ui, "\u{21BB}", "Rotate", EditorTool::Rotate);
                    self.draw_tool_button(ui, "\u{25A1}", "Scale", EditorTool::Scale);

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    let (pr, resp) =
                        ui.allocate_exact_size(egui::vec2(64.0, 24.0), egui::Sense::click());
                    if ui.is_rect_visible(pr) {
                        let fill = if self.playing {
                            egui::Color32::from_rgb(0xCC, 0x44, 0x44)
                        } else {
                            egui::Color32::from_rgb(0x3A, 0x8C, 0x4A)
                        };
                        ui.painter().rect(
                            pr,
                            egui::CornerRadius::same(4),
                            fill,
                            egui::Stroke::NONE,
                            egui::StrokeKind::Inside,
                        );
                        let icon = if self.playing { "\u{25A0}" } else { "\u{25B6}" };
                        let label = if self.playing { " Stop" } else { " Play" };
                        ui.painter().text(
                            pr.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{}{}", icon, label),
                            egui::FontId::proportional(13.0),
                            egui::Color32::WHITE,
                        );
                    }
                    if resp.clicked() {
                        self.playing = !self.playing;
                    }
                });
            });
    }

    fn draw_tool_button(&mut self, ui: &mut egui::Ui, icon: &str, label: &str, tool: EditorTool) {
        let selected = self.current_tool == tool;
        let (rect, response) = ui.allocate_exact_size(egui::vec2(32.0, 22.0), egui::Sense::click());
        if ui.is_rect_visible(rect) {
            let fill = if selected {
                crate::theme::accent_color()
            } else if response.hovered() {
                ui.style().visuals.widgets.hovered.bg_fill
            } else {
                ui.style().visuals.widgets.inactive.bg_fill
            };
            ui.painter().rect(
                rect,
                egui::CornerRadius::same(4),
                fill,
                egui::Stroke::NONE,
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
            label,
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
                ui.label(
                    egui::RichText::new("Scene Hierarchy")
                        .color(crate::theme::text_bright())
                        .size(13.0)
                        .strong(),
                );
                ui.add_space(6.0);

                egui::ScrollArea::vertical().id_salt("h_scroll").show(ui, |ui| {
                    if self.scene_entities.is_empty() {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("  (empty scene)")
                                .color(crate::theme::text_dim())
                                .size(12.0),
                        );
                        return;
                    }
                    for &entity in &self.scene_entities {
                        let name = self.get_entity_name(entity);
                        let selected = self.selected_entity == Some(entity);
                        let icon = if name.contains("Sphere") {
                            "\u{25C9}"
                        } else if name.contains("Grid") {
                            "\u{25A6}"
                        } else {
                            "\u{25A0}"
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
            });
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
                ui.label(
                    egui::RichText::new("Inspector")
                        .color(crate::theme::text_bright())
                        .size(13.0)
                        .strong(),
                );
                ui.add_space(6.0);

                if let Some(entity) = self.selected_entity {
                    let name = self.get_entity_name(entity);
                    let t = self.get_entity_transform(entity);
                    egui::ScrollArea::vertical().id_salt("insp_scroll").show(ui, |ui| {
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(&name)
                                .color(crate::theme::text_bright())
                                .size(14.0)
                                .strong(),
                        );
                        ui.add_space(8.0);

                        let bg = crate::theme::faint_bg_color();
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 60.0),
                            egui::Sense::hover(),
                        );
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
                        for (label, val) in
                            [("X", t.translation.x), ("Y", t.translation.y), ("Z", t.translation.z)]
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

                        ui.add_space(8.0);
                        let (rect2, _) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 40.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect(
                            rect2,
                            egui::CornerRadius::same(4),
                            bg,
                            egui::Stroke::NONE,
                            egui::StrokeKind::Inside,
                        );
                        let cr2 = rect2.shrink(8.0);
                        let mut y2 = cr2.min.y + 4.0;
                        ui.painter().text(
                            egui::pos2(cr2.min.x, y2),
                            egui::Align2::LEFT_TOP,
                            "Rotation",
                            egui::FontId::proportional(11.0),
                            crate::theme::text_dim(),
                        );
                        y2 += 16.0;
                        let euler = t.rotation.to_euler(glam::EulerRot::YXZ);
                        ui.painter().text(
                            egui::pos2(cr2.min.x, y2),
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

                        ui.add_space(8.0);
                        let has_rotator = self.world.get::<Rotator>(entity).is_some();
                        ui.label(format!("Rotator: {}", if has_rotator { "ON" } else { "off" }));
                    });
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

    fn build_status_bar(&mut self, _ctx: &egui::Context) {
        let state = if self.playing { "PLAYING" } else { "EDIT" };
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
                        egui::RichText::new(format!(
                            "SchiroEngine v0.1  |  {}  |  {}",
                            state, tool_name
                        ))
                        .color(crate::theme::text_dim())
                        .size(11.0),
                    );
                });
            });
    }

    fn build_editor_ui(&mut self, ctx: &egui::Context) {
        self.build_menu_bar(ctx);
        self.build_hierarchy_panel(ctx);
        self.build_inspector_panel(ctx);
        self.build_status_bar(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(0)))
            .show(ctx, |ui| {
                let vf = egui::Frame::none()
                    .fill(crate::theme::faint_bg_color())
                    .stroke(egui::Stroke::new(1.0_f32, crate::theme::faint_bg_color()))
                    .inner_margin(egui::Margin::same(0));
                vf.show(ui, |ui| {
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
}

impl ApplicationHandler for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_title("SchiroEngine Editor")
            .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080))
            .with_maximized(true);
        let window = Arc::new(event_loop.create_window(attributes).unwrap());
        let ws = window.inner_size();
        let mut renderer =
            pollster::block_on(Renderer::new(Arc::clone(&window), (ws.width, ws.height)))
                .expect("failed to create renderer");

        init_scene(
            &mut self.world,
            &mut renderer,
            &self.asset_server,
            &mut self.scene_entities,
            &mut self.entity_mesh_map,
            &mut self.gizmo_mesh_start,
        );

        self.egui_winit_state = Some(egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::default(),
            &window,
            None,
            None,
            None,
        ));
        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: winit::window::WindowId,
        event: WindowEvent,
    ) {
        self.input.process_window_event(&event);
        if let (Some(s), Some(w)) = (self.egui_winit_state.as_mut(), self.window.as_ref()) {
            let _ = s.on_window_event(w, &event);
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.render_frame(),
            WindowEvent::Resized(ps) => {
                if let Some(r) = self.renderer.as_mut() {
                    r.resize(ps.width, ps.height);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(w) = self.window.as_ref() {
            w.request_redraw();
        }
    }
}

fn init_scene(
    world: &mut World,
    renderer: &mut Renderer,
    asset_server: &schiro_assets::AssetServer,
    entities: &mut Vec<Entity>,
    mesh_map: &mut HashMap<Entity, usize>,
    gizmo_start: &mut usize,
) {
    let gizmo = schiro_render::GizmoMeshes::new();
    *gizmo_start = renderer.mesh_count();
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
    let sphere = asset_server.load("proc://sphere", |_| Ok(sphere_asset.clone())).unwrap();
    let sm = asset_to_render_mesh(&sphere);
    let t = Mat4::from_translation(Vec3::new(0.0, 1.5, 0.0));
    let mi = renderer.mesh_count();
    renderer.add_mesh(&sm, &t);

    let entity = world
        .spawn((
            schiro_ecs::components::Name("Sphere".into()),
            Transform { translation: Vec3::new(0.0, 1.5, 0.0), ..Default::default() },
            schiro_ecs::components::GlobalTransform::default(),
            MeshRenderer { mesh_handle: Some(mi), visible: true },
            Rotator { speed: Vec3::new(0.0, 1.5, 0.0) },
        ))
        .id();
    entities.push(entity);
    mesh_map.insert(entity, mi);

    let gm = schiro_render::Mesh::grid(10, 10, 1.0);
    let gi = renderer.mesh_count();
    renderer.add_mesh(&gm, &Mat4::IDENTITY);
    let ge = world
        .spawn((
            schiro_ecs::components::Name("Grid".into()),
            Transform::default(),
            schiro_ecs::components::GlobalTransform::default(),
            MeshRenderer { mesh_handle: Some(gi), visible: true },
        ))
        .id();
    entities.push(ge);
    mesh_map.insert(ge, gi);

    info!("scene: {} entities", entities.len());
}

fn update_gizmo_transforms(
    renderer: &mut Renderer,
    world: &World,
    selected: Option<Entity>,
    gizmo_start: usize,
    tool: EditorTool,
) {
    let hide = Mat4::from_scale(Vec3::ZERO);
    if let Some(entity) = selected {
        let pos = world.get::<Transform>(entity).map(|t| t.translation).unwrap_or(Vec3::ZERO);
        let t = Mat4::from_translation(pos);
        let (tr, rr, sr) = match tool {
            EditorTool::Translate => (0..6, 6..6, 9..9),
            EditorTool::Rotate => (0..0, 6..9, 9..9),
            EditorTool::Scale => (0..0, 6..6, 9..12),
        };
        for i in 0..12 {
            let idx = gizmo_start + i;
            let show = (i >= tr.start as usize && i < tr.end as usize)
                || (i >= rr.start as usize && i < rr.end as usize)
                || (i >= sr.start as usize && i < sr.end as usize);
            renderer.update_mesh_transform(idx, if show { &t } else { &hide });
        }
    } else {
        for i in 0..12 {
            renderer.update_mesh_transform(gizmo_start + i, &hide);
        }
    }
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
