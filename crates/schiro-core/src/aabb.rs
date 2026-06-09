//! Axis aligned bounding boxes and infinite rays used for spatial queries
//! such as picking and culling.
//!
//! These types are kept intentionally minimal: they are POD and do not
//! reference any rendering or physics data. Higher level crates compose
//! them into more elaborate hierarchies.

use glam::Vec3;

/// Axis aligned bounding box defined by a minimum and a maximum corner.
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    /// Minimum corner of the box, in object or world space.
    pub min: Vec3,
    /// Maximum corner of the box, in object or world space.
    pub max: Vec3,
}

impl Aabb {
    /// Box with both corners at the origin. Useful as a default value.
    pub const ZERO: Self = Self {
        min: Vec3::ZERO,
        max: Vec3::ZERO,
    };

    /// Builds a box from explicit minimum and maximum corners.
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Computes the smallest axis aligned box containing the supplied set
    /// of points.
    ///
    /// Returns a degenerate box at the origin if the slice is empty.
    pub fn from_points(points: &[[f32; 3]]) -> Self {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for p in points {
            let v = Vec3::from(*p);
            min = min.min(v);
            max = max.max(v);
        }
        Self { min, max }
    }

    /// Returns the center of the box.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Returns the size (extent) of the box along each axis.
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Returns the radius of the smallest sphere that contains the box.
    pub fn radius(&self) -> f32 {
        self.size().length() * 0.5
    }

    /// Returns a new box transformed by the supplied matrix.
    ///
    /// The eight corners are transformed individually and the result is
    /// the axis aligned bounding volume of those points. This is not the
    /// tightest possible oriented bounding box.
    pub fn transform(&self, mat: &glam::Mat4) -> Self {
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];
        let t: Vec<Vec3> = corners
            .iter()
            .map(|c| mat.transform_point3(*c))
            .collect();
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for p in t {
            min = min.min(p);
            max = max.max(p);
        }
        Self { min, max }
    }
}

/// Ray defined by an origin and a normalized direction.
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    /// Origin of the ray, in world space.
    pub origin: Vec3,
    /// Normalized direction the ray points toward.
    pub direction: Vec3,
}

impl Ray {
    /// Builds a ray and normalizes the direction.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Returns the point at distance `t` along the ray.
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    /// Returns the smallest non-negative `t` at which the ray enters the
    /// box, or `None` if the ray misses the box entirely.
    pub fn intersects_aabb(&self, aabb: &Aabb) -> Option<f32> {
        let t1 = (aabb.min - self.origin) / self.direction;
        let t2 = (aabb.max - self.origin) / self.direction;
        let tmin = t1.min(t2);
        let tmax = t1.max(t2);
        let near = tmin.x.max(tmin.y).max(tmin.z);
        let far = tmax.x.min(tmax.y).min(tmax.z);

        if near <= far && far >= 0.0 {
            Some(near.max(0.0))
        } else {
            None
        }
    }
}
