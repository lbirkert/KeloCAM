use nalgebra::Vector3;

pub struct Ray {
    normal: Vector3<f32>,
    origin: Vector3<f32>,
}

impl Ray {
    pub fn new(origin: Vector3<f32>, normal: Vector3<f32>) -> Self {
        assert!((1.0 - normal.dot(&normal)).abs() < 0.001);
        Self { origin, normal }
    }

    pub fn plane_intersect(
        &self,
        plane_origin: Vector3<f32>,
        plane_normal: Vector3<f32>,
    ) -> Option<Vector3<f32>> {
        let a = plane_normal.dot(&self.normal);

        // Ray and plane parallel
        if a == 0.0 {
            return None;
        }

        let t = (plane_origin - self.origin).dot(&plane_normal) / a;

        // Plane is behind ray
        if t < 0.0 {
            return None;
        }

        Some(self.origin + self.normal.scale(t))
    }

    pub fn triangle_intersect(
        &self,
        triangle_a: Vector3<f32>,
        triangle_b: Vector3<f32>,
        triangle_c: Vector3<f32>,
        triangle_normal: Vector3<f32>,
    ) -> Option<Vector3<f32>> {
        let point = self.plane_intersect(triangle_a, triangle_normal);

        if point.is_none() {
            return None;
        }

        let point = point.unwrap();

        let ab = triangle_a - triangle_b;
        let ac = triangle_a - triangle_c;
        let pa = point - triangle_a;
        let pb = point - triangle_b;
        let pc = point - triangle_c;

        let two_area_abc = ab.cross(&ac).magnitude();

        let alpha = pb.cross(&pc).magnitude() / two_area_abc;
        let beta = pc.cross(&pa).magnitude() / two_area_abc;
        let gamma = 1.0 - alpha - beta;

        if alpha >= 0.0
            && alpha <= 1.0
            && beta >= 0.0
            && beta <= 1.0
            && gamma >= 0.0
            && gamma <= 1.0
        {
            Some(point)
        } else {
            None
        }
    }
}
