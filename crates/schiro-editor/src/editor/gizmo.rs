use bevy_ecs::prelude::*;
use egui::Key;
use glam::{Vec2, Vec3};
use schiro_core::{Aabb, Ray};
use schiro_ecs::components::Transform;

use crate::app::{EditorApp, EditorTool, GizmoDrag};

pub fn handle_viewport_input(app: &mut EditorApp, aspect: f32) {
    let vp_size = egui::vec2(app.viewport_panel.rect.width(), app.viewport_panel.rect.height());

    for key in app.viewport_panel.keys_pressed.clone() {
        match key {
            Key::W => app.current_tool = EditorTool::Translate,
            Key::E => app.current_tool = EditorTool::Rotate,
            Key::R => app.current_tool = EditorTool::Scale,
            _ => {}
        }
    }

    app.viewport_panel.gizmo_hovered = check_gizmo_hover(app, aspect);

    let clicked = app.viewport_panel.clicked_pos.take();

    if app.viewport_panel.gizmo_held && app.gizmo_drag.is_none() {
        if let Some(entity) = app.selected_entity {
            if let Some((mx, my)) = app.viewport_panel.gizmo_press_pos {
                let ray = app.viewport_panel.camera.screen_to_ray(
                    Vec2::new(mx, my), Vec2::new(vp_size.x, vp_size.y), aspect);
                let pos = app.get_entity_transform(entity).translation;
                let (axis, _) = pick_gizmo_axis(&ray, pos, app.current_tool);
                if axis < 3 { app.gizmo_drag = Some(GizmoDrag { axis, entity }); }
            }
        }
    }

    if !app.viewport_panel.gizmo_held { app.gizmo_drag = None; }

    if let Some((x, y)) = clicked {
        let ray = app.viewport_panel.camera.screen_to_ray(Vec2::new(x, y), Vec2::new(vp_size.x, vp_size.y), aspect);
        let mut best: Option<(Entity, f32)> = None;
        for &entity in &app.scene_entities {
            let pos = app.get_entity_transform(entity).translation;
            let aabb = Aabb::new(pos - Vec3::splat(0.5), pos + Vec3::splat(0.5));
            if let Some(t) = ray.intersects_aabb(&aabb) {
                if best.map_or(true, |(_, d)| t < d) { best = Some((entity, t)); }
            }
        }
        app.selected_entity = best.map(|(e, _)| e);
    }

    if let Some(drag) = app.gizmo_drag {
        let (dx, dy) = app.viewport_panel.mouse_delta;
        if dx != 0.0 || dy != 0.0 {
            match app.current_tool {
                EditorTool::Translate => drag_translate(app, drag, dx, dy),
                EditorTool::Rotate => drag_rotate(app, drag, dx),
                EditorTool::Scale => drag_scale(app, drag, dx),
            }
        }
    }
}

fn check_gizmo_hover(app: &EditorApp, aspect: f32) -> bool {
    let Some(entity) = app.selected_entity else { return false };
    let pos = app.get_entity_transform(entity).translation;
    let vp_size = egui::vec2(app.viewport_panel.rect.width(), app.viewport_panel.rect.height());
    if vp_size.x <= 0.0 || vp_size.y <= 0.0 { return false; }
    let (mx, my) = (app.viewport_panel.prev_mouse.0, app.viewport_panel.prev_mouse.1);
    if mx == 0.0 && my == 0.0 { return false; }
    let ray = app.viewport_panel.camera.screen_to_ray(Vec2::new(mx, my), Vec2::new(vp_size.x, vp_size.y), aspect);
    let (axis, _) = pick_gizmo_axis(&ray, pos, app.current_tool);
    axis < 3
}

fn drag_translate(app: &mut EditorApp, drag: GizmoDrag, dx: f32, dy: f32) {
    let cam = &app.viewport_panel.camera;
    let right = cam.view_matrix().inverse().x_axis.truncate();
    let up = cam.view_matrix().inverse().y_axis.truncate();
    let world_delta = right * dx * cam.distance * 0.002 + up * (-dy) * cam.distance * 0.002;
    let axis_dir = match drag.axis { 0 => Vec3::X, 1 => Vec3::Y, 2 => Vec3::Z, _ => return };
    if let Some(mut t) = app.world.get_mut::<Transform>(drag.entity) {
        t.translation += axis_dir * world_delta.dot(axis_dir);
    }
}

fn drag_rotate(app: &mut EditorApp, drag: GizmoDrag, dx: f32) {
    let axis = match drag.axis { 0 => Vec3::X, 1 => Vec3::Y, 2 => Vec3::Z, _ => return };
    let rot = glam::Quat::from_axis_angle(axis, dx * 0.01);
    if let Some(mut t) = app.world.get_mut::<Transform>(drag.entity) {
        t.rotation = rot * t.rotation;
    }
}

fn drag_scale(app: &mut EditorApp, drag: GizmoDrag, dx: f32) {
    let factor = 1.0 + dx * 0.01;
    if let Some(mut t) = app.world.get_mut::<Transform>(drag.entity) {
        match drag.axis { 0 => t.scale.x *= factor, 1 => t.scale.y *= factor, 2 => t.scale.z *= factor, _ => {} }
    }
}

pub fn pick_gizmo_axis(ray: &Ray, center: Vec3, tool: EditorTool) -> (usize, f32) {
    match tool {
        EditorTool::Translate => {
            let hw = 0.04; let th = 0.08;
            for (i, dir) in [Vec3::X, Vec3::Y, Vec3::Z].iter().enumerate() {
                if let Some(t) = ray.intersects_aabb(&aabb_along_axis(center, *dir, 0.0, 0.8, hw)) { return (i, t); }
                if let Some(t) = ray.intersects_aabb(&aabb_along_axis(center, *dir, 0.8, 0.2, th)) { return (i, t); }
            }
        }
        EditorTool::Rotate => {
            for (i, axis) in [Vec3::X, Vec3::Y, Vec3::Z].iter().enumerate() {
                if let Some(t) = ray.intersects_aabb(&ring_aabb(center, *axis, 1.0, 0.05)) { return (i, t); }
            }
        }
        EditorTool::Scale => {
            for (i, dir) in [Vec3::X, Vec3::Y, Vec3::Z].iter().enumerate() {
                if let Some(t) = ray.intersects_aabb(&aabb_along_axis(center, *dir, 0.83, 0.14, 0.07)) { return (i, t); }
            }
        }
    }
    (3, f32::MAX)
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
    if axis == Vec3::X { min.x = center.x - thickness; max.x = center.x + thickness; }
    if axis == Vec3::Y { min.y = center.y - thickness; max.y = center.y + thickness; }
    if axis == Vec3::Z { min.z = center.z - thickness; max.z = center.z + thickness; }
    Aabb::new(min, max)
}
