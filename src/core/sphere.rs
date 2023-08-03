use nalgebra::Vector3;

pub struct Sphere {
    pub origin: Vector3<f32>,
    pub radius: f32,
}

impl Sphere {
    pub fn new(origin: Vector3<f32>, radius: f32) -> Self {
        Self { origin, radius }
    }
}
