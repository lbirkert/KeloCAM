use nalgebra::Vector3;

#[derive(Clone, Debug)]
pub enum Trans {
    Translate(Vector3<f32>),
    Rotate(Vector3<f32>),
    Scale(f32),
    ScaleNonUniformly(Vector3<f32>),
}
