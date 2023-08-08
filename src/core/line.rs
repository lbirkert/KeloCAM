use nalgebra::Vector3;

use super::Plane;

#[derive(Debug)]
pub struct Line {
    pub a: Vector3<f32>,
    pub b: Vector3<f32>,
}

impl Line {
    pub fn new(a: Vector3<f32>, b: Vector3<f32>) -> Self {
        Self { a, b }
    }

    /// Returns the intersection of this line with a plane (if any).
    pub fn intersect_plane(&self, plane: &Plane) -> Option<Vector3<f32>> {
        Self::intersect_plane_raw(&self.a, &self.b, plane)
    }

    /// Returns the intersection of a line with start and endpoints on a plane (if any).
    pub fn intersect_plane_raw(
        a: &Vector3<f32>,
        b: &Vector3<f32>,
        plane: &Plane,
    ) -> Option<Vector3<f32>> {
        let s = (a - b).dot(&plane.normal);

        // => line and plane parallel
        if s == 0.0 {
            return None;
        }

        let t = (plane.origin - b).dot(&plane.normal) / s;

        // => point not on line
        if !(0.0..1.0).contains(&t) {
            return None;
        }

        Some(b.lerp(a, t))
    }
}
