use nalgebra::{UnitVector3, Vector3};

use super::ray::Ray;

pub struct Plane {
    pub origin: Vector3<f32>,
    pub normal: UnitVector3<f32>,
}

impl Plane {
    pub fn new(origin: Vector3<f32>, normal: UnitVector3<f32>) -> Self {
        Self { normal, origin }
    }

    /// Perform a ray intersection with this plane.
    /// Returns the intersection point if any, otherwise None.
    pub fn intersect(&self, ray: &Ray) -> Option<Vector3<f32>> {
        Self::intersect_raw(&self.origin, &self.normal, ray)
    }

    /// Perform a ray intersection with this plane.
    /// Returns the intersection point if any, otherwise None.
    pub fn intersect_raw(
        origin: &Vector3<f32>,
        normal: &UnitVector3<f32>,
        ray: &Ray,
    ) -> Option<Vector3<f32>> {
        let a = normal.dot(&ray.normal);

        // Ray and plane parallel
        if a == 0.0 {
            return None;
        }

        let t = (origin - ray.origin).dot(&normal) / a;

        // Plane is behind ray
        if t < 0.0 {
            return None;
        }

        Some(ray.origin + ray.normal.scale(t))
    }
}
