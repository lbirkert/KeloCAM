use nalgebra::{UnitVector3, Vector3};

use super::plane::Plane;

#[derive(Debug)]
pub struct Ray {
    pub normal: Vector3<f32>,
    pub origin: Vector3<f32>,
}

impl Ray {
    pub fn new(origin: Vector3<f32>, normal: Vector3<f32>) -> Self {
        assert!((1.0 - normal.dot(&normal)).abs() < 0.001);
        Self { origin, normal }
    }

    pub fn circle_intersect(
        &self,
        circle_origin: &Vector3<f32>,
        circle_normal: &UnitVector3<f32>,
        circle_radius: &std::ops::Range<f32>,
    ) -> Option<Vector3<f32>> {
        self.circle_intersect_squared(
            circle_origin,
            circle_normal,
            (circle_radius.start * circle_radius.start)..(circle_radius.end * circle_radius.end),
        )
    }

    pub fn circle_intersect_squared(
        &self,
        circle_origin: &Vector3<f32>,
        circle_normal: &UnitVector3<f32>,
        circle_radius_squared: std::ops::Range<f32>,
    ) -> Option<Vector3<f32>> {
        let p = Plane::intersect_raw(circle_origin, circle_normal, self)?;

        if circle_radius_squared.contains(&(p - circle_origin).magnitude_squared()) {
            Some(p)
        } else {
            None
        }
    }
}
