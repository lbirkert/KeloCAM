use nalgebra::Vector3;

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

    pub fn plane_intersect(
        &self,
        plane_origin: &Vector3<f32>,
        plane_normal: &Vector3<f32>,
    ) -> Option<Vector3<f32>> {
        let a = plane_normal.dot(&self.normal);

        // Ray and plane parallel
        if a == 0.0 {
            return None;
        }

        let t = (plane_origin - self.origin).dot(plane_normal) / a;

        // Plane is behind ray
        if t < 0.0 {
            return None;
        }

        Some(self.origin + self.normal.scale(t))
    }

    // Algorithm from https://math.stackexchange.com/questions/4322/check-whether-a-point-is-within-a-3d-triangle#28552
    pub fn triangle_intersect(
        &self,
        a: &Vector3<f32>,
        b: &Vector3<f32>,
        c: &Vector3<f32>,
        triangle_normal: &Vector3<f32>,
    ) -> Option<Vector3<f32>> {
        let p = self.plane_intersect(a, triangle_normal)?;

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

    pub fn square_intersect(
        &self,
        a: &Vector3<f32>,
        ab: &Vector3<f32>,
        ac: &Vector3<f32>,
    ) -> Option<Vector3<f32>> {
        let p = self.plane_intersect(a, &ab.cross(ac).normalize())?;

        let ap = p - a;

        if (0.0..=ab.magnitude_squared()).contains(&ap.dot(ab))
            && (0.0..=ac.magnitude_squared()).contains(&ap.dot(ac))
        {
            Some(p)
        } else {
            None
        }
    }
}
