use nalgebra::{UnitVector3, Vector3};

use super::{Geometry, Ray};

#[derive(Debug)]
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
    pub fn intersect_ray_raw(
        origin: &Vector3<f32>,
        normal: &UnitVector3<f32>,
        ray: &Ray,
    ) -> Option<Vector3<f32>> {
        let a = normal.dot(&ray.normal);

        // Ray and plane parallel
        if a == 0.0 {
            return None;
        }

        let t = (origin - ray.origin).dot(normal) / a;

        // Plane is behind ray
        if t < 0.0 {
            return None;
        }

        Some(ray.origin + ray.normal.scale(t))
    }

    pub fn intersect<T>(&self, entity: &T) -> Option<Vector3<f32>>
    where
        T: PlaneIntersection,
    {
        entity.intersect_plane(self)
    }
}

pub trait PlaneIntersection {
    fn intersect_plane_raw(
        &self,
        origin: &Vector3<f32>,
        normal: &UnitVector3<f32>,
    ) -> Option<Vector3<f32>>;
    fn intersect_plane(&self, plane: &Plane) -> Option<Vector3<f32>> {
        self.intersect_plane_raw(&plane.origin, &plane.normal)
    }
}

impl Geometry for Plane {}
