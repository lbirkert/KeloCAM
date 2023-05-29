use crate::object::{point::Point2D};
use std::collections::HashMap;

type BoundingBox2D = (Point2D, Point2D);

#[derive(Debug, Clone)]
pub enum Segment {
    Line { start: Point2D, end: Point2D },
    Arc { center: Point2D, rad: f64, start: f64, end: f64 },
}

impl Segment {
    pub fn bounding_box(&self) -> BoundingBox2D {
        match self {
            Self::Line { start, end } => (start.min(*end), start.max(*end)),
            Self::Arc { center, start, end, rad } => {
                const HALF_PI: f64 = std::f64::consts::PI / 2.0;

                let start_field = (start / HALF_PI).floor();
                let end_field = (end / HALF_PI).floor();

                let fields = end_field - start_field;

                let max_y: f64 = if fields >= 4.0-(start_field-0.0).rem_euclid(4.0) || fields < -(start_field-0.0).rem_euclid(4.0)
                    { 1.0 } else { start.cos().max(end.cos()) };
                let max_x: f64 = if fields >= 4.0-(start_field-1.0).rem_euclid(4.0) || fields < -(start_field-1.0).rem_euclid(4.0)
                    { 1.0 } else { start.sin().max(end.sin()) };
                let min_y: f64 = if fields >= 4.0-(start_field-2.0).rem_euclid(4.0) || fields < -(start_field-2.0).rem_euclid(4.0)
                    { -1.0 } else { start.cos().min(end.cos()) };
                let min_x: f64 = if fields >= 4.0-(start_field-3.0).rem_euclid(4.0) || fields < -(start_field-3.0).rem_euclid(4.0)
                    { -1.0 } else { start.sin().min(end.sin()) };

                (
                    Point2D::from(min_x, min_y) * rad + center,
                    Point2D::from(max_x, max_y) * rad + center,
                )
            }
        }
    }

    pub fn start(&self) -> Point2D {
        match self {
            Self::Line { start, .. } => start.clone(),
            Self::Arc { center, start, rad, .. } => Point2D::from(start.sin(), start.cos()) * rad + center,
        }
    }    

    pub fn end(&self) -> Point2D {
        match self {
            Self::Line { end, .. } => end.clone(),
            Self::Arc { center, end, rad, .. } => Point2D::from(end.sin(), end.cos()) * rad + center,
        }
    }
}

#[derive(Debug)]
struct ContourSegment {
    segment: Segment,
    // Hash of next segment
    next: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Contour {
    pub segments: Vec<Segment>,
    pub index: u8,
}

impl Contour {
    pub fn bounding_box(&self) -> BoundingBox2D {
        let mut min: Option<Point2D> = None;
        let mut max: Option<Point2D> = None;

        for segment in &self.segments {
            let bounding = segment.bounding_box();

            if let Some(ref mins) = min {
                min = Some(mins.min(bounding.0));
            } else {
                min = Some(bounding.0);
            }
            
            if let Some(ref maxs) = max {
                max = Some(maxs.max(bounding.1));
            } else {
                max = Some(bounding.1);
            }
        }

        return (min.unwrap(), max.unwrap());
    }

    pub fn find(segments: Vec<Segment>) -> Vec<Self> {
        let mut csegments: HashMap<usize, ContourSegment> = segments
            .clone()
            .into_iter()
            .enumerate()
            .map(|(a_hash, a)| {
                let a_end = &a.end();

                let mut next = None;
                for b_hash in 0..segments.len() {
                    let b = &segments[b_hash];
                    
                    let b_start = &b.start();
                    
                    if a_end == b_start {
                        next = Some(b_hash);
                        break;
                    }
                }

                (a_hash, ContourSegment {
                    segment: a,
                    next
                })
            })
            .collect();
    
        let mut contours = Vec::new();
        
        // Create contours
        for cseg in 0..csegments.len() {
            let mut segments = Vec::new();

            if let Some(mut current) = csegments.remove(&cseg) {
                while let Some(next) = current.next {
                    segments.push(current.segment);
                    
                    if next == cseg {
                        contours.push(Contour {
                            index: 0,
                            segments,
                        });

                        break;
                    }

                    current = csegments.remove(&next).unwrap();
                }
            } else {
                continue;
            }
        }

        contours
    }

    pub fn index(contours: Vec<Self>) -> Vec<Self> {
        contours
            .clone()
            .into_iter()
            .map(|mut a| {
                let a_b = &a.bounding_box();

                for b_i in 0..contours.len() {
                    let b_b = contours[b_i].bounding_box();

                    // Check if bounding box of a is contained inside bounding box of a
                    if b_b.0.smaller(a_b.0) && b_b.1.greater(a_b.1) {
                        a.index += 1;
                    }
                }

                a
            })
            .collect()
    }
}

