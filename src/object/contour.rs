use crate::object::{point::Point2D};

pub enum Segment {
    Line { start: Point2D, end: Point2D },
    Arc { center: Point2D, rad: f64, start: f64, end: f64 },
}

impl Segment {
    pub fn bounding_box(self) -> (Point2D, Point2D) {
        match self {
            Self::Line { start, end } => (start.min(end), start.max(end)),
            Self::Arc { center, start, end, rad } => {
                const HALF_PI: f64 = std::f64::consts::PI / 2.0;

                let start_field = (start / HALF_PI).floor();
                let end_field = (end / HALF_PI).floor();

                let fields = end_field - start_field;

                let max_y: f64 = if fields >= 4.0 -((start_field - 0.0) % 4.0) || fields < -((start_field - 0.0) % 4.0)
                    { 1.0 } else { start.cos().max(end.cos()) };
                let max_x: f64 = if fields >= 4.0 -((start_field - 1.0) % 4.0) || fields < -((start_field - 1.0) % 4.0)
                    { 1.0 } else { start.sin().max(end.sin()) };
                let min_y: f64 = if fields >= 4.0 -((start_field - 2.0) % 4.0) || fields < -((start_field - 2.0) % 4.0)
                    { -1.0 } else { start.cos().min(end.cos()) };
                let min_x: f64 = if fields >= 4.0 -((start_field - 3.0) % 4.0) || fields < -((start_field - 3.0) % 4.0)
                    { -1.0 } else { start.sin().min(end.sin()) };

                (
                    Point2D::from(min_x, min_y) * rad + center,
                    Point2D::from(max_x, max_y) * rad + center,
                )
            }
        }
    }

    pub fn start(self) -> Point2D {
        match self {
            Self::Line { start, .. } => start.clone(),
            Self::Arc { center, start, rad, .. } => Point2D::from(start.sin(), start.cos()) * rad + center,
        }
    }    

    pub fn end(self) -> Point2D {
        match self {
            Self::Line { end, .. } => end.clone(),
            Self::Arc { center, end, rad, .. } => Point2D::from(end.sin(), end.cos()) * rad + center,
        }
    }
}

pub struct Contour {
    index: u8,
    segments: Vec<Segment>,
    min: Point2D,
    max: Point2D,
}

impl Contour {
    pub fn bounding_box(segments: Vec<Segment>) -> (Point2D, Point2D) {
        let mut min: Option<Point2D> = None;
        let mut max: Option<Point2D> = None;
        
        for segment in segments {
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

    pub fn find(entities: Vec<Segment>) -> Vec<Self> {
        let contours = Vec::new();

        for _entity in entities {
         
        }

        contours
    }
}

