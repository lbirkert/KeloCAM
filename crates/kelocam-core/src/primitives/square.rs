use nalgebra::{UnitVector3, Vector3};

use super::{ray::RayIntersection, BoundingBox, Geometry, Plane, Ray};

#[derive(Debug)]
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
    pub fn intersect_ray_raw(
        a: &Vector3<f32>,
        ab: &Vector3<f32>,
        ac: &Vector3<f32>,
        normal: &UnitVector3<f32>,
        ray: &Ray,
    ) -> Option<Vector3<f32>> {
        let p = Plane::intersect_ray_raw(a, normal, ray)?;

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

impl RayIntersection for Square {
    fn intersect_ray(&self, ray: &Ray) -> Option<Vector3<f32>> {
        Self::intersect_ray_raw(&self.a, &self.ab, &self.ac, &self.normal, ray)
    }
}

impl BoundingBox for Square {
    fn bb_min(&self) -> Vector3<f32> {
        let a = self.a;
        let b = self.a + self.ab;
        let c = self.a + self.ac;
        let d = self.a + self.ab + self.ac;
        a.inf(&b.inf(&c.inf(&d)))
    }

    fn bb_max(&self) -> Vector3<f32> {
        let a = self.a;
        let b = self.a + self.ab;
        let c = self.a + self.ac;
        let d = self.a + self.ab + self.ac;
        a.sup(&b.sup(&c.sup(&d)))
    }
}

impl Geometry for Square {}
