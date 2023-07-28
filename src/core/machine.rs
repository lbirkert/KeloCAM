use nalgebra::Vector3;

pub struct Machine {
    pub name: String,
    pub dimensions: Vector3<f32>,
}

impl Machine {
    pub fn new(name: String, dimensions: Vector3<f32>) -> Self {
        Self { name, dimensions }
    }
}
