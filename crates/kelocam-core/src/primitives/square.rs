use nalgebra::{UnitVector3, Vector3};

use super::{plane::PlaneIntersection, BoundingBox, Geometry};

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

    /// Perform an intersection with this square.
    /// Returns the intersection point if any, otherwise None.
    pub fn intersect_raw<T>(
        a: &Vector3<f32>,
        ab: &Vector3<f32>,
        ac: &Vector3<f32>,
        normal: &UnitVector3<f32>,
        entity: &T,
    ) -> Option<Vector3<f32>>
    where
        T: PlaneIntersection,
    {
        let p = entity.intersect_plane_raw(a, normal)?;

        let ap = p - a;

        if (0.0..=ab.magnitude_squared()).contains(&ap.dot(ab))
            && (0.0..=ac.magnitude_squared()).contains(&ap.dot(ac))
        {
            Some(p)
        } else {
            None
        }
    }

    /// Perform an intersection with this square.
    /// Returns the intersection point if any, otherwise None.
    pub fn intersect<T>(&self, entity: &T) -> Option<Vector3<f32>>
    where
        T: PlaneIntersection,
    {
        Self::intersect_raw(&self.a, &self.ab, &self.ac, &self.normal, entity)
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
