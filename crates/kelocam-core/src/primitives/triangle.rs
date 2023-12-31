use nalgebra::{UnitVector3, Vector3};

use super::{plane::PlaneIntersection, BoundingBox, Geometry};

#[derive(Debug, Clone)]
pub struct Triangle {
    pub a: Vector3<f32>,
    pub b: Vector3<f32>,
    pub c: Vector3<f32>,
    pub normal: UnitVector3<f32>,
}

impl Triangle {
    pub fn new(
        a: Vector3<f32>,
        b: Vector3<f32>,
        c: Vector3<f32>,
        normal: UnitVector3<f32>,
    ) -> Self {
        Self { a, b, c, normal }
    }

    pub fn from_stl(stl: stl::Triangle) -> Self {
        Self::new(
            Vector3::from(stl.v1).scale(0.1),
            Vector3::from(stl.v2).scale(0.1),
            Vector3::from(stl.v3).scale(0.1),
            UnitVector3::new_normalize(Vector3::from(stl.normal)),
        )
    }

    /// Perform a ray intersection with this triangle.
    /// Returns the intersection point if any, otherwise None.
    pub fn intersect_raw<T>(
        a: &Vector3<f32>,
        b: &Vector3<f32>,
        c: &Vector3<f32>,
        normal: &UnitVector3<f32>,
        entity: &T,
    ) -> Option<Vector3<f32>>
    where
        T: PlaneIntersection,
    {
        // Algorithm from https://math.stackexchange.com/questions/4322/check-whether-a-point-is-within-a-3d-triangle#28552
        let p = entity.intersect_plane_raw(a, normal)?;

        let n = (b - a).cross(&(c - a));
        let nl = n.magnitude_squared();
        let n_a = (c - b).cross(&(p - b));
        let n_b = (a - c).cross(&(p - c));
        let n_c = (b - a).cross(&(p - a));

        let alpha = n.dot(&n_a) / nl;
        let beta = n.dot(&n_b) / nl;
        let gamma = n.dot(&n_c) / nl;

        if (0.0..=1.0).contains(&alpha)
            && (0.0..=1.0).contains(&beta)
            && (0.0..=1.0).contains(&gamma)
        {
            Some(p)
        } else {
            None
        }
    }

    /// Perform a ray intersection with this triangle.
    /// Returns the intersection point if any, otherwise None.
    pub fn intersect<T>(&self, entity: &T) -> Option<Vector3<f32>>
    where
        T: PlaneIntersection,
    {
        Self::intersect_raw(&self.a, &self.b, &self.c, &self.normal, entity)
    }
}

impl BoundingBox for Triangle {
    fn bb_min(&self) -> Vector3<f32> {
        self.a.inf(&self.b.inf(&self.c))
    }

    fn bb_max(&self) -> Vector3<f32> {
        self.a.sup(&self.b.sup(&self.c))
    }
}

impl Geometry for Triangle {}
