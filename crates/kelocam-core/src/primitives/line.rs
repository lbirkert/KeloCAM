use nalgebra::{UnitVector3, Vector3};

use super::{plane::PlaneIntersection, BoundingBox, Geometry};

#[derive(Debug)]
pub struct Line {
    pub a: Vector3<f32>,
    pub b: Vector3<f32>,
}

impl Line {
    pub fn new(a: Vector3<f32>, b: Vector3<f32>) -> Self {
        Self { a, b }
    }
}

impl PlaneIntersection for Line {
    /// Returns the intersection of a line with start and endpoints on a plane (if any).
    fn intersect_plane_raw(
        &self,
        origin: &Vector3<f32>,
        normal: &UnitVector3<f32>,
    ) -> Option<Vector3<f32>> {
        let s = (self.a - self.b).dot(normal);

        // => line and plane parallel
        if s == 0.0 {
            return None;
        }

        let t = (origin - self.b).dot(normal) / s;

        // => point not on line
        if !(0.0..=1.0).contains(&t) {
            return None;
        }

        Some(self.b.lerp(&self.a, t))
    }
}

impl BoundingBox for Line {
    fn bb_min(&self) -> Vector3<f32> {
        self.a.inf(&self.b)
    }

    fn bb_max(&self) -> Vector3<f32> {
        self.a.sup(&self.b)
    }
}

impl Geometry for Line {}
