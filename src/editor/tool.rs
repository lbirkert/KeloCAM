use nalgebra::{UnitVector3, Vector3};

use crate::core::primitives::{Axis, Ray, Square};

/// An action state a tool can be in.
pub enum Action {
    Hover(Axis),
    Transform(Axis),
}

pub enum Tool {
    Move,
    Scale { uniform: bool },
    Rotate,
}

impl Tool {
    pub fn intersect_axis(
        origin: &Vector3<f32>,
        axis: &Axis,
        scale: f32,
        ray: &Ray,
    ) -> Option<Vector3<f32>> {
        let eye_normal = ray.origin - origin;

        let mut ortho = eye_normal.cross(&axis.vector);
        ortho.normalize_mut();
        ortho.scale_mut(scale * 0.2);

        let normal = UnitVector3::new_normalize(axis.vector.cross(&ortho));

        Square::intersect_raw(
            &(origin - ortho.scale(0.5)),
            &ortho,
            &axis.vector.scale(scale),
            &normal,
            &ray,
        )
    }

    pub fn intersect(&self, origin: &Vector3<f32>, scale: f32, ray: &Ray) -> Option<Axis> {
        match self {
            Tool::Move | Tool::Scale { .. } => {
                let mut axis = None;

                if Self::intersect_axis(&origin, &Axis::X, scale, ray).is_some() {
                    axis = Some(Axis::X);
                }
                if Self::intersect_axis(&origin, &Axis::Y, scale, ray).is_some() {
                    axis = Some(Axis::Y);
                }
                if Self::intersect_axis(&origin, &Axis::Z, scale, ray).is_some() {
                    axis = Some(Axis::Z);
                }

                axis
            }
            Tool::Rotate => {
                const TOLLERANCE: f32 = 0.2;
                let radius = ((1.0 - TOLLERANCE) * scale)..((1.0 + TOLLERANCE) * scale);

                let mut axis = None;
                let mut dist = std::f32::INFINITY;

                if let Some(p) = ray.circle_intersect(origin, &Vector3::x_axis(), &radius) {
                    dist = (p - ray.origin).magnitude_squared();
                    axis = Some(Axis::X);
                }
                if let Some(p) = ray.circle_intersect(origin, &Vector3::y_axis(), &radius) {
                    let pdist = (p - ray.origin).magnitude_squared();
                    if pdist < dist {
                        axis = Some(Axis::Y);
                        dist = pdist;
                    }
                }
                if let Some(p) = ray.circle_intersect(origin, &Vector3::z_axis(), &radius) {
                    if (p - ray.origin).magnitude_squared() < dist {
                        axis = Some(Axis::Z);
                    }
                }

                axis
            }
        }
    }
}
