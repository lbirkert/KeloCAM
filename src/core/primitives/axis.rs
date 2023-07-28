use nalgebra::{UnitVector3, Vector3};

pub struct Axis {
    pub vector: UnitVector3<f32>,
}

impl Axis {
    pub const X: Axis = Axis {
        vector: UnitVector3::new_unchecked(Vector3::new(1.0, 0.0, 0.0)),
    };
    pub const Y: Axis = Axis {
        vector: UnitVector3::new_unchecked(Vector3::new(0.0, 1.0, 0.0)),
    };
    pub const Z: Axis = Axis {
        vector: UnitVector3::new_unchecked(Vector3::new(0.0, 0.0, 1.0)),
    };
}
