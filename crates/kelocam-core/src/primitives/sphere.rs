use nalgebra::Vector3;

use super::{BoundingBox, Geometry};

#[derive(Debug)]
pub struct Sphere {
    pub origin: Vector3<f32>,
    pub radius: f32,
}

impl Sphere {
    pub fn new(origin: Vector3<f32>, radius: f32) -> Self {
        Self { origin, radius }
    }
}

impl BoundingBox for Sphere {
    fn bb_min(&self) -> Vector3<f32> {
        self.origin - Vector3::from_element((self.radius * self.radius / 3.0).sqrt())
    }

    fn bb_max(&self) -> Vector3<f32> {
        self.origin + Vector3::from_element((self.radius * self.radius / 3.0).sqrt())
    }
}

impl Geometry for Sphere {}
