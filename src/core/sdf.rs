use nalgebra::Vector2;

use super::Path2;

// Is the contour of a SDF (sign distance field) of some object at a certain distance.
#[derive(Debug)]
pub struct Contour {
    segments: Vec<Segment>,
    distance: f32,
}

impl Contour {
    pub fn new(segments: Vec<Segment>, distance: f32) -> Self {
        Self { segments, distance }
    }

    pub fn path(&self) -> Path2 {
        let mut points = Vec::new();

        for segment in self.segments.iter() {
            match segment {
                Segment::Line { to, .. } => points.push(*to),
                Segment::Arc {
                    start,
                    delta,
                    center,
                } => {
                    let res: usize = (delta.abs() * 2.0).ceil() as usize;

                    for i in 0..=res {
                        let angle = start + (i as f32 / res as f32) * delta;
                        let (sin, cos) = angle.sin_cos();
                        points.push(center + Vector2::new(sin, cos).scale(self.distance));
                    }
                }
            };
        }

        Path2::new(points)
    }
}

#[derive(Debug)]
pub enum Segment {
    Arc {
        start: f32,
        delta: f32,
        center: Vector2<f32>,
    },
    Line {
        from: Vector2<f32>,
        to: Vector2<f32>,
    },
}
