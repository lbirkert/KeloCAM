use nalgebra::{UnitVector3, Vector3};

use super::{plane::Plane, ray::Ray};

pub struct Square {
    pub a: Vector3<f32>,
    pub ab: Vector3<f32>,
    pub ac: Vector3<f32>,
    pub normal: UnitVector3<f32>,
}

impl Square {
    pub fn new(
        a: Vector3<f32>,
        ab: Vector3<f32>,
        ac: Vector3<f32>,
        normal: UnitVector3<f32>,
    ) -> Self {
        Self { a, ab, ac, normal }
    }

    /// Perform a ray intersection with this square.
    /// Returns the intersection point if any, otherwise None.
    pub fn intersect(&self, ray: &Ray) -> Option<Vector3<f32>> {
        Self::intersect_raw(&self.a, &self.ab, &self.ac, &self.normal, ray)
    }

    /// Perform a ray intersection with this square.
    /// Returns the intersection point if any, otherwise None.
    pub fn intersect_raw(
        a: &Vector3<f32>,
        ab: &Vector3<f32>,
        ac: &Vector3<f32>,
        normal: &UnitVector3<f32>,
        ray: &Ray,
    ) -> Option<Vector3<f32>> {
        let p = Plane::intersect_raw(a, normal, ray)?;

        let ap = p - a;

        if (0.0..=ab.magnitude_squared()).contains(&ap.dot(ab))
            && (0.0..=ac.magnitude_squared()).contains(&ap.dot(ac))
        {
            Some(p)
        } else {
            None
        }
    }
}
