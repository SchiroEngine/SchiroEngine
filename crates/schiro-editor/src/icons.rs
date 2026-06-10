//! Geometric entity icons drawn with the egui painter.

use egui::{Color32, CornerRadius, Pos2, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Cube,
    Sphere,
    Plane,
    Light,
    Empty,
    Mesh,
    ToolTranslate,
    ToolRotate,
    ToolScale,
    Wireframe,
    Play,
    Stop,
}

pub fn icon_for_entity(name: &str) -> Icon {
    if name.contains("Cube") {
        Icon::Cube
    } else if name.contains("Sphere") {
        Icon::Sphere
    } else if name.contains("Plane") {
        Icon::Plane
    } else if name.contains("Light") {
        Icon::Light
    } else if name.contains("Grid") {
        Icon::Mesh
    } else {
        Icon::Empty
    }
}

pub fn draw_icon(painter: &egui::Painter, rect: Rect, icon: Icon) {
    let size = 12.0;
    let cx = rect.left() + size * 0.5 + 6.0;
    let cy = rect.center().y;
    let half = size * 0.5;

    match icon {
        Icon::Cube => {
            let cuboid = Color32::from_rgb(0xCC, 0x88, 0x44);
            let area =
                Rect::from_center_size(Pos2::new(cx, cy), egui::vec2(size - 4.0, size - 4.0));
            painter.rect_filled(area, CornerRadius::same(2), cuboid);
        }
        Icon::Sphere => {
            let sphere = Color32::from_rgb(0x66, 0x99, 0xCC);
            painter.circle_filled(Pos2::new(cx, cy), half - 2.0, sphere);
        }
        Icon::Plane => {
            let plane = Color32::from_rgb(0x77, 0xAA, 0x77);
            let area = Rect::from_center_size(Pos2::new(cx, cy), egui::vec2(size - 5.0, 3.0));
            painter.rect_filled(area, CornerRadius::same(1), plane);
        }
        Icon::Light => {
            let light = Color32::from_rgb(0xEE, 0xCC, 0x44);
            painter.circle_filled(Pos2::new(cx, cy), half - 2.0, light);
        }
        Icon::Empty => {
            painter.circle_stroke(
                Pos2::new(cx, cy),
                half - 2.0,
                egui::Stroke::new(1.0, Color32::from_rgb(0x88, 0x88, 0x8C)),
            );
        }
        Icon::Mesh => {
            let mesh_col = Color32::from_rgb(0x99, 0x99, 0xAA);
            for &(dx, dy) in &[(-3.0, -3.0), (3.0, -3.0), (-3.0, 3.0), (3.0, 3.0)] {
                painter.circle_filled(Pos2::new(cx + dx, cy + dy), 1.5, mesh_col);
            }
        }
        Icon::ToolTranslate => {
            let col = Color32::from_rgb(0xCC, 0xCC, 0xCC);
            let s = 3.0;
            painter.line_segment(
                [Pos2::new(cx - s, cy), Pos2::new(cx + s, cy)],
                egui::Stroke::new(1.5, col),
            );
            painter.line_segment(
                [Pos2::new(cx, cy - s), Pos2::new(cx, cy + s)],
                egui::Stroke::new(1.5, col),
            );
            let a = 1.5;
            painter.line_segment(
                [Pos2::new(cx + s, cy), Pos2::new(cx + s - a, cy - a)],
                egui::Stroke::new(1.5, col),
            );
            painter.line_segment(
                [Pos2::new(cx + s, cy), Pos2::new(cx + s - a, cy + a)],
                egui::Stroke::new(1.5, col),
            );
            painter.line_segment(
                [Pos2::new(cx, cy - s), Pos2::new(cx - a, cy - s + a)],
                egui::Stroke::new(1.5, col),
            );
            painter.line_segment(
                [Pos2::new(cx, cy - s), Pos2::new(cx + a, cy - s + a)],
                egui::Stroke::new(1.5, col),
            );
        }
        Icon::ToolRotate => {
            let col = Color32::from_rgb(0xCC, 0xCC, 0xCC);
            let r = half - 2.0;
            let n = 16;
            let start = -0.8;
            let end = 2.6;
            let mut pts = Vec::new();
            for i in 0..=n {
                let a = start + (end - start) * i as f32 / n as f32;
                pts.push(Pos2::new(cx + r * a.cos(), cy - r * a.sin()));
            }
            for w in pts.windows(2) {
                painter.line_segment([w[0], w[1]], egui::Stroke::new(1.5, col));
            }
            let tip = pts.last().copied().unwrap();
            let ang = end + std::f32::consts::FRAC_PI_2 * 0.7;
            let a = 2.5;
            painter.line_segment(
                [tip, Pos2::new(tip.x + a * ang.cos(), tip.y - a * ang.sin())],
                egui::Stroke::new(1.5, col),
            );
        }
        Icon::ToolScale => {
            let col = Color32::from_rgb(0xCC, 0xCC, 0xCC);
            let s = half - 2.0;
            painter.rect_stroke(
                Rect::from_center_size(Pos2::new(cx, cy), egui::vec2(s * 1.4, s * 1.4)),
                CornerRadius::same(1),
                egui::Stroke::new(1.5, col),
                egui::StrokeKind::Inside,
            );
            let a = 2.0;
            painter.line_segment(
                [Pos2::new(cx + s * 0.7, cy - s * 0.7), Pos2::new(cx + s * 0.7 + a, cy - s * 0.7)],
                egui::Stroke::new(1.5, col),
            );
            painter.line_segment(
                [Pos2::new(cx + s * 0.7, cy - s * 0.7), Pos2::new(cx + s * 0.7, cy - s * 0.7 - a)],
                egui::Stroke::new(1.5, col),
            );
        }
        Icon::Wireframe => {
            let col = Color32::from_rgb(0xBB, 0xBB, 0xBB);
            let s = half - 2.0;
            let r = Rect::from_center_size(Pos2::new(cx, cy), egui::vec2(s * 1.6, s * 1.6));
            painter.rect_stroke(
                r,
                CornerRadius::same(1),
                egui::Stroke::new(1.2, col),
                egui::StrokeKind::Inside,
            );
            painter.line_segment(
                [Pos2::new(r.left(), r.top()), Pos2::new(r.right(), r.bottom())],
                egui::Stroke::new(1.2, col),
            );
        }
        Icon::Play => {
            let col = Color32::from_rgb(0x33, 0xAA, 0x44);
            let s = half - 2.0;
            let pts = [
                Pos2::new(cx - s * 0.6, cy - s),
                Pos2::new(cx + s, cy),
                Pos2::new(cx - s * 0.6, cy + s),
            ];
            painter.add(egui::Shape::convex_polygon(pts.to_vec(), col, egui::Stroke::NONE));
        }
        Icon::Stop => {
            let col = Color32::from_rgb(0xCC, 0x33, 0x33);
            let s = half - 2.0;
            let r = Rect::from_center_size(Pos2::new(cx, cy), egui::vec2(s * 1.4, s * 1.4));
            painter.rect_filled(r, CornerRadius::same(1), col);
        }
    }
}
