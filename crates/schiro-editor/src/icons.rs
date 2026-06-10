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
    }
}
