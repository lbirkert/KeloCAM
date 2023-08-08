use nalgebra::{UnitVector3, Vector3};

#[derive(Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub const X_VEC: UnitVector3<f32> = UnitVector3::new_unchecked(Vector3::new(1.0, 0.0, 0.0));
    pub const Y_VEC: UnitVector3<f32> = UnitVector3::new_unchecked(Vector3::new(0.0, 1.0, 0.0));
    pub const Z_VEC: UnitVector3<f32> = UnitVector3::new_unchecked(Vector3::new(0.0, 0.0, 1.0));

    pub fn vector(&self) -> &'static UnitVector3<f32> {
        match self {
            Self::X => &Self::X_VEC,
            Self::Y => &Self::Y_VEC,
            Self::Z => &Self::Z_VEC,
        }
    }
}
