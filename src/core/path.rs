use nalgebra::{UnitVector2, UnitVector3, Vector2, Vector3};

use super::sdf;

#[derive(Debug)]
pub struct Path2 {
    points: Vec<Vector2<f32>>,
}

impl Path2 {
    pub fn new(points: Vec<Vector2<f32>>) -> Self {
        Self { points }
    }

    /// Creates a new sanitized 2D path
    pub fn new_sanitize(points: Vec<Vector2<f32>>) -> Self {
        let mut path = Self::new(points);
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

            const EPSILON: f32 = 1e-3;

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

    pub fn sdf_contour(&self, distance: f32) -> sdf::Contour {
        let len = self.points.len();

        let mut segments: Vec<sdf::Segment> = Vec::new();
        let mut points: Vec<Vector2<f32>> = Vec::new();

        for i in 0..len {
            let a = &self.points[if i > 0 { i } else { len } - 1];
            let b = &self.points[i];
            let c = &self.points[(i + 1) % len];

            let mut d1 = a - b;
            let mut d2 = b - c;
            d1.normalize_mut();
            d2.normalize_mut();

            let n1 = Vector2::new(d1.y, -d1.x);
            let n2 = Vector2::new(d2.y, -d2.x);

            if distance.signum() * n2.dot(&d1) > 0.0 {
                let n = UnitVector2::new_normalize(n1 + n2);
                let l = distance / n.dot(&n1);

                let side = l * n.dot(&d1);
                let side_squared = side * side;

                if side_squared < (a - b).magnitude_squared()
                    && side_squared < (b - c).magnitude_squared()
                {
                    let from = b + n.scale(l);
                    points.push(from);

                    // to will be filled after all points have been constructed
                    segments.push(sdf::Segment::Line {
                        from,
                        to: Vector2::zeros(),
                    });
                }
            } else {
                let start = n1.x.signum() * Vector2::y_axis().dot(&n1).acos();
                let delta = -d1.dot(&n2).signum() * n1.dot(&n2).acos();

                points.push(b + n1.scale(distance));

                segments.push(sdf::Segment::Arc {
                    start,
                    delta,
                    center: *b,
                });
                // to will be filled after all points have been constructed
                segments.push(sdf::Segment::Line {
                    from: b + n2.scale(distance),
                    to: Vector2::zeros(),
                });
            }
        }

        let mut i = 0;
        for segment in segments.iter_mut() {
            if let sdf::Segment::Line { ref mut to, .. } = segment {
                i += 1;
                *to = points[i % points.len()];
            }
        }

        sdf::Contour::new(segments, distance)
    }
}

#[derive(Debug)]
pub struct Path3 {
    pub points: Vec<Vector3<f32>>,
}

impl Path3 {
    pub fn new(points: Vec<Vector3<f32>>) -> Self {
        Self { points }
    }

    /// Creates a new sanitized 3D path
    pub fn new_sanitize(points: Vec<Vector3<f32>>) -> Self {
        let mut path = Self::new(points);
        path.sanitize();
        path
    }

    /// Sanitize this path. This will delete all points whose left and right edge have the
    /// same normal vector rounded to EPSILON (~ 1e-3)
    pub fn sanitize(&mut self) {
        let mut i = 0;
        while i < self.points.len() {
            let len = self.points.len();

            let a = &self.points[if i > 0 { i } else { len } - 1];
            let b = &self.points[i];
            let c = &self.points[(i + 1) % len];

            const EPSILON: f32 = 1e-3;

            if ((a - b).normalize() - (b - c).normalize()).magnitude_squared() < EPSILON {
                self.points.remove(i);
            } else {
                i += 1;
            }
        }
    }
}
