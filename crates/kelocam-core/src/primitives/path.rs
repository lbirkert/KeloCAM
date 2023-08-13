use nalgebra::Vector3;

use super::{BoundingBox, Geometry};

#[derive(Debug)]
pub struct Path3 {
    pub points: Vec<Vector3<f32>>,
}

impl Path3 {
    pub fn new(points: Vec<Vector3<f32>>) -> Self {
        Self { points }
    }

    /// Creates a new sanitized 3D path
    pub fn new_sanitize(points: Vec<Vector3<f32>>) -> Self {
        let mut path = Self::new(points);
        path.sanitize();
        path
    }

    /// Sanitize this path. This will delete all points whose left and right edge have the
    /// same normal vector rounded to EPSILON (~ 1e-3)
    pub fn sanitize(&mut self) {
        let mut i = 0;
        while i < self.points.len() {
            let len = self.points.len();

            let a = &self.points[if i > 0 { i } else { len } - 1];
            let b = &self.points[i];
            let c = &self.points[(i + 1) % len];

            const EPSILON: f32 = 1e-3;

            if ((a - b).normalize() - (b - c).normalize()).magnitude_squared() < EPSILON {
                self.points.remove(i);
            } else {
                i += 1;
            }
        }
    }
}

impl BoundingBox for Path3 {
    fn bb_min(&self) -> Vector3<f32> {
        let mut min = Vector3::from_element(std::f32::INFINITY);
        for point in self.points.iter() {
            min = min.inf(point);
        }
        min
    }

    fn bb_max(&self) -> Vector3<f32> {
        let mut max = Vector3::from_element(std::f32::NEG_INFINITY);
        for point in self.points.iter() {
            max = max.inf(point);
        }
        max
    }
}

impl Geometry for Path3 {}
