use nalgebra::{UnitVector3, Vector2, Vector3};

pub struct Path2 {
    pub points: Vec<Vector2<f32>>,
}

impl Path2 {
    pub fn new(points: Vec<Vector2<f32>>) -> Self {
        Self { points }
    }

    /// Extends this path to a 3D version.
    /// Generates the points via `offset + x_normal * x + y_normal * y`.
    pub fn extend3(
        &self,
        offset: &Vector3<f32>,
        x_normal: &UnitVector3<f32>,
        y_normal: &UnitVector3<f32>,
    ) -> Path3 {
        let mut points =
            Vec::with_capacity(self.points.len() * std::mem::size_of::<Vector3<f32>>());

        for point in self.points.iter() {
            points.push(offset + x_normal.scale(point.x) + y_normal.scale(point.y));
        }

        Path3::new(points)
    }
}

pub struct Path3 {
    pub points: Vec<Vector3<f32>>,
}

impl Path3 {
    pub fn new(points: Vec<Vector3<f32>>) -> Self {
        Self { points }
    }
}
