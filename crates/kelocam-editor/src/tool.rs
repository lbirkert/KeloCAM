use nalgebra::{UnitVector3, Vector3};

use crate::renderer;
use kelocam_core::{Axis, Mesh, Ray, Square};

/// An action state a tool can be in.
pub enum Action {
    Hover(Axis),
    Transform(Axis),
}

pub enum Tool {
    Translate(Vector3<f32>),
    Scale(f32),
    ScaleNonUniformly(Vector3<f32>),
    Rotate(Vector3<f32>),
}

impl Tool {
    pub fn reset(&mut self) {
        match self {
            Tool::Translate(ref mut delta) => *delta = Vector3::zeros(),
            Tool::Rotate(ref mut delta) => *delta = Vector3::zeros(),
            Tool::Scale(ref mut delta) => *delta = 1.0,
            Tool::ScaleNonUniformly(ref mut delta) => *delta = Vector3::from_element(1.0),
        }
    }

    pub fn apply(&self, mesh: &mut Mesh) {
        match self {
            Tool::Translate(delta) => mesh.translate(delta),
            Tool::Scale(delta) => mesh.scale(*delta),
            Tool::ScaleNonUniformly(delta) => mesh.scale_non_uniformly(delta),
            Tool::Rotate(delta) => mesh.rotate(delta),
        }
    }

    pub fn translate() -> Self {
        Self::Translate(Vector3::zeros())
    }

    pub fn scale() -> Self {
        Self::Scale(1.0)
    }

    pub fn scale_non_uniformly() -> Self {
        Self::ScaleNonUniformly(Vector3::from_element(1.0))
    }

    pub fn rotate() -> Self {
        Self::Rotate(Vector3::zeros())
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
        let mut xcolor = [1.0, 0.0, 0.0, 1.0];
        let mut ycolor = [0.0, 1.0, 0.0, 1.0];
        let mut zcolor = [0.0, 0.0, 1.0, 1.0];
        if let Some(action) = action {
            match action {
                Action::Hover(Axis::X) | Action::Transform(Axis::X) => {
                    xcolor = [1.0, 0.6, 0.6, 1.0]
                }
                Action::Hover(Axis::Y) | Action::Transform(Axis::Y) => {
                    ycolor = [0.6, 1.0, 0.6, 1.0]
                }
                Action::Hover(Axis::Z) | Action::Transform(Axis::Z) => {
                    zcolor = [0.6, 0.6, 1.0, 1.0]
                }
            };
        }

        match self {
            Self::Translate { .. } | Self::Scale { .. } | Self::ScaleNonUniformly { .. } => {
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
                    Self::Translate { .. } => {
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
                    Self::Scale { .. } | Self::ScaleNonUniformly { .. } => {
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
            Self::Rotate { .. } => {
                let mut xp = Vec::new();
                let mut yp = Vec::new();
                let mut zp = Vec::new();
                const RES: i32 = 30;
                for i in 0..RES {
                    let angle = (i as f32 / RES as f32) * std::f32::consts::TAU;
                    let (mut sin, mut cos) = angle.sin_cos();
                    sin *= scale;
                    cos *= scale;
                    xp.push(Vector3::new(0.0, sin, cos) + origin);
                    yp.push(Vector3::new(sin, 0.0, cos) + origin);
                    zp.push(Vector3::new(sin, cos, 0.0) + origin);
                }

                renderer::path::generate_closed(&xp, xcolor, 0.01, path_verticies, path_indicies);
                renderer::entity::generate_arrow(
                    0.1 * scale,
                    &(origin + Vector3::new(0.0, 0.1 * scale, 0.98 * scale)),
                    &Vector3::y_axis(),
                    xcolor,
                    entity_verticies,
                );
                renderer::entity::generate_arrow(
                    0.1 * scale,
                    &(origin + Vector3::new(0.0, -0.1 * scale, 0.98 * scale)),
                    &-Vector3::y_axis(),
                    xcolor,
                    entity_verticies,
                );

                renderer::path::generate_closed(&yp, ycolor, 0.01, path_verticies, path_indicies);
                renderer::entity::generate_arrow(
                    0.1 * scale,
                    &(origin + Vector3::new(0.98 * scale, 0.0, 0.1 * scale)),
                    &Vector3::z_axis(),
                    ycolor,
                    entity_verticies,
                );
                renderer::entity::generate_arrow(
                    0.1 * scale,
                    &(origin + Vector3::new(0.98 * scale, 0.0, -0.1 * scale)),
                    &-Vector3::z_axis(),
                    ycolor,
                    entity_verticies,
                );

                renderer::path::generate_closed(&zp, zcolor, 0.01, path_verticies, path_indicies);
                renderer::entity::generate_arrow(
                    0.1 * scale,
                    &(origin + Vector3::new(0.1 * scale, 0.98 * scale, 0.0)),
                    &Vector3::x_axis(),
                    zcolor,
                    entity_verticies,
                );
                renderer::entity::generate_arrow(
                    0.1 * scale,
                    &(origin + Vector3::new(-0.1 * scale, 0.98 * scale, 0.0)),
                    &-Vector3::x_axis(),
                    zcolor,
                    entity_verticies,
                );
            }
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

        Square::intersect_ray_raw(
            &(origin - ortho.scale(0.5)),
            &ortho,
            &axis.vector().scale(scale),
            &normal,
            ray,
        )
    }

    pub fn intersect(&self, origin: &Vector3<f32>, scale: f32, ray: &Ray) -> Option<Axis> {
        match self {
            Tool::Translate(_) | Tool::Scale(_) | Tool::ScaleNonUniformly(_) => {
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
            Tool::Rotate(_) => {
                const TOLLERANCE: f32 = 0.1;
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

impl Default for Tool {
    fn default() -> Self {
        Self::translate()
    }
}
