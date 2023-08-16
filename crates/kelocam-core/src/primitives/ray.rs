use nalgebra::{UnitVector3, Vector3};

use super::{plane::PlaneIntersection, Plane};

#[derive(Debug)]
pub struct Ray {
    pub normal: UnitVector3<f32>,
    pub origin: Vector3<f32>,
}

impl Ray {
    pub fn new(origin: Vector3<f32>, normal: UnitVector3<f32>) -> Self {
        Self { origin, normal }
    }

    pub fn circle_intersect(
        &self,
        circle_origin: &Vector3<f32>,
        circle_normal: &UnitVector3<f32>,
        circle_radius: &std::ops::Range<f32>,
    ) -> Option<Vector3<f32>> {
        self.circle_intersect_squared(
            circle_origin,
            circle_normal,
            (circle_radius.start * circle_radius.start)..(circle_radius.end * circle_radius.end),
        )
    }

    pub fn circle_intersect_squared(
        &self,
        circle_origin: &Vector3<f32>,
        circle_normal: &UnitVector3<f32>,
        circle_radius_squared: std::ops::Range<f32>,
    ) -> Option<Vector3<f32>> {
        let p = Plane::intersect_ray_raw(circle_origin, circle_normal, self)?;

        if circle_radius_squared.contains(&(p - circle_origin).magnitude_squared()) {
            Some(p)
        } else {
            None
        }
    }
}

impl PlaneIntersection for Ray {
    fn intersect_plane_raw(
        &self,
        origin: &Vector3<f32>,
        normal: &UnitVector3<f32>,
    ) -> Option<Vector3<f32>> {
        let a = normal.dot(&self.normal);

        // Ray and plane parallel
        if a == 0.0 {
            return None;
        }

        let t = (origin - self.origin).dot(normal) / a;

        // Plane is behind ray
        if t < 0.0 {
            return None;
        }

        Some(self.origin + self.normal.scale(t))
    }
}

// Works like a ray but also in the other direction
#[derive(Debug)]
pub struct InfiniteLine {
    pub normal: UnitVector3<f32>,
    pub origin: Vector3<f32>,
}

impl InfiniteLine {
    pub fn new(origin: Vector3<f32>, normal: UnitVector3<f32>) -> Self {
        Self { origin, normal }
    }
}

impl PlaneIntersection for InfiniteLine {
    fn intersect_plane_raw(
        &self,
        origin: &Vector3<f32>,
        normal: &UnitVector3<f32>,
    ) -> Option<Vector3<f32>> {
        let a = normal.dot(&self.normal);

        // Line and plane parallel
        if a == 0.0 {
            return None;
        }

        let t = (origin - self.origin).dot(normal) / a;

        Some(self.origin + self.normal.scale(t))
    }
}
