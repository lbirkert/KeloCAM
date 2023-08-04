use nalgebra::{UnitVector2, UnitVector3, Vector2, Vector3};

pub struct Path2 {
    points: Vec<Vector2<f32>>,
}

impl Path2 {
    /// Creates a new sanitized path instance.
    pub fn new(points: Vec<Vector2<f32>>) -> Self {
        let mut path = Self { points };
        path.sanitize();
        path
    }

    /// Sanitize this path. This will delete all points whose left and right edge have the
    /// same normal vector rounded to EPSILON (~ 1e-2)
    pub fn sanitize(&mut self) {
        let mut i = 0;
        while i < self.points.len() {
            let len = self.points.len();

            let a = &self.points[if i > 0 { i } else { len } - 1];
            let b = &self.points[i];
            let c = &self.points[(i + 1) % len];

            const EPSILON: f32 = 1e-2;

            if ((a - b).normalize() - (b - c).normalize()).magnitude_squared() < EPSILON {
                self.points.remove(i);
            } else {
                i += 1;
            }
        }
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

    /// Extrudes the edges along their normals (e.g. outwards) by factor.
    pub fn extrude(&self, factor: f32) -> Path2 {
        let len = self.points.len();

        let mut points = Vec::new();

        for i in 0..len {
            let a = &self.points[if i > 0 { i } else { len } - 1];
            let b = &self.points[i];
            let c = &self.points[(i + 1) % len];

            let mut n1 = Vector2::new(a.y - b.y, b.x - a.x);
            let mut n2 = Vector2::new(b.y - c.y, c.x - b.x);
            n1.normalize_mut();
            n2.normalize_mut();

            let n = UnitVector2::new_normalize(n1 + n2);
            let l = factor / n.dot(&n1);

            points.push(b - n.scale(l));
        }

        println!("{points:?} {}", points.len());

        Path2::new(points)
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
