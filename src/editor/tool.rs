use nalgebra::{UnitVector3, Vector3};

use crate::{
    core::primitives::{Axis, Ray, Square, Trans},
    renderer,
};

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
    /// Return the default transformation for this tool, which when applied does not change the
    /// vertex positions and vertex normals.
    pub fn default_trans(&self) -> Trans {
        match self {
            Self::Move => Trans::Translate(Vector3::zeros()),
            Self::Scale { uniform } => {
                if *uniform {
                    Trans::Scale(1.0)
                } else {
                    Trans::ScaleNonUniformly(Vector3::from_element(1.0))
                }
            }
            Self::Rotate => Trans::Rotate(Vector3::zeros()),
        }
    }

    /// Generate the UI part of this tool.
    pub fn generate(
        &self,
        origin: &Vector3<f32>,
        scale: f32,
        action: &Option<Action>,
        path_verticies: &mut Vec<renderer::path::Vertex>,
        path_indicies: &mut Vec<renderer::path::Index>,
        entity_verticies: &mut Vec<renderer::entity::Vertex>,
    ) {
        let mut xcolor = [1.0, 0.0, 0.0];
        let mut ycolor = [0.0, 1.0, 0.0];
        let mut zcolor = [0.0, 0.0, 1.0];
        if let Some(action) = action {
            match action {
                Action::Hover(Axis::X) | Action::Transform(Axis::X) => xcolor = [1.0, 0.4, 0.4],
                Action::Hover(Axis::Y) | Action::Transform(Axis::Y) => ycolor = [0.4, 1.0, 0.4],
                Action::Hover(Axis::Z) | Action::Transform(Axis::Z) => zcolor = [0.4, 0.4, 1.0],
            };
        }

        match self {
            Self::Move | Self::Scale { .. } => {
                let x_end = Vector3::new(scale, 0.0, 0.0) + origin;
                let y_end = Vector3::new(0.0, scale, 0.0) + origin;
                let z_end = Vector3::new(0.0, 0.0, scale) + origin;
                renderer::path::generate_open(
                    &[*origin, x_end],
                    xcolor,
                    0.01,
                    path_verticies,
                    path_indicies,
                );
                renderer::path::generate_open(
                    &[*origin, y_end],
                    ycolor,
                    0.01,
                    path_verticies,
                    path_indicies,
                );
                renderer::path::generate_open(
                    &[*origin, z_end],
                    zcolor,
                    0.01,
                    path_verticies,
                    path_indicies,
                );

                match self {
                    Self::Move => {
                        renderer::entity::generate_arrow(
                            scale * 0.1,
                            &x_end,
                            &Vector3::x_axis(),
                            xcolor,
                            entity_verticies,
                        );
                        renderer::entity::generate_arrow(
                            scale * 0.1,
                            &y_end,
                            &Vector3::y_axis(),
                            ycolor,
                            entity_verticies,
                        );
                        renderer::entity::generate_arrow(
                            scale * 0.1,
                            &z_end,
                            &Vector3::z_axis(),
                            zcolor,
                            entity_verticies,
                        );
                    }
                    Self::Scale { .. } => {
                        renderer::entity::generate_cube(
                            scale * 0.1,
                            &x_end,
                            xcolor,
                            entity_verticies,
                        );
                        renderer::entity::generate_cube(
                            scale * 0.1,
                            &y_end,
                            ycolor,
                            entity_verticies,
                        );
                        renderer::entity::generate_cube(
                            scale * 0.1,
                            &z_end,
                            zcolor,
                            entity_verticies,
                        );
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn intersect_axis(
        origin: &Vector3<f32>,
        axis: &Axis,
        scale: f32,
        ray: &Ray,
    ) -> Option<Vector3<f32>> {
        let eye_normal = ray.origin - origin;

        let mut ortho = eye_normal.cross(axis.vector());
        ortho.normalize_mut();
        ortho.scale_mut(scale * 0.2);

        let normal = UnitVector3::new_normalize(ortho.cross(axis.vector()));

        Square::intersect_raw(
            &(origin - ortho.scale(0.5)),
            &ortho,
            &axis.vector().scale(scale),
            &normal,
            ray,
        )
    }

    pub fn intersect(&self, origin: &Vector3<f32>, scale: f32, ray: &Ray) -> Option<Axis> {
        match self {
            Tool::Move | Tool::Scale { .. } => {
                let mut axis = None;

                if Self::intersect_axis(origin, &Axis::X, scale, ray).is_some() {
                    axis = Some(Axis::X);
                }
                if Self::intersect_axis(origin, &Axis::Y, scale, ray).is_some() {
                    axis = Some(Axis::Y);
                }
                if Self::intersect_axis(origin, &Axis::Z, scale, ray).is_some() {
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
