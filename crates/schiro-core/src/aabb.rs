use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub const ZERO: Self = Self {
        min: Vec3::ZERO,
        max: Vec3::ZERO,
    };

    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

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

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn radius(&self) -> f32 {
        self.size().length() * 0.5
    }

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

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

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
