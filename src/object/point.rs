use std::ops;

const EQUAL_THRESHOLD: f64 = 0.0000001;

#[macro_export]
macro_rules! p2 {
    { $A:expr, $B:expr } => { kelocam::object::point::Point2D::from(($A) as f64, ($B) as f64) };
}

#[derive(Debug, Copy, Clone)]
pub struct Point2D {
    x: f64,
    y: f64,
}

impl Point2D {
    pub fn from(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn min(self, b: Self) -> Self {
        Point2D::from(self.x.min(b.x), self.y.min(b.y))
    }
    
    pub fn max(self, b: Self) -> Self {
        Point2D::from(self.x.max(b.x), self.y.max(b.y))
    }

    pub fn dist(self, b: Self) -> f64 {
        (self - b).len()
    }

    pub fn len(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    
    pub fn dist_approx(self, b: Self) -> f64 {
        (self - b).len_approx()
    }
    
    pub fn len_approx(self) -> f64 {
        self.x.abs() + self.y.abs()
    }

    pub fn greater(self, b: Self) -> bool {
        self.x > b.x && self.y > b.y
    }

    pub fn smaller(self, b: Self) -> bool {
        self.x < b.x && self.y < b.y
    }
}

impl ops::Add for Point2D {
    type Output = Self;
    
    fn add(self, b: Self) -> Self {
        Point2D::from(self.x + b.x, self.y + b.y)
    }
}

impl ops::Add<&Point2D> for Point2D {
    type Output = Self;
    
    fn add(self, b: &Self) -> Self {
        Point2D::from(self.x + b.x, self.y + b.y)
    }
}

impl ops::Sub for Point2D {
    type Output = Self;
    
    fn sub(self, b: Self) -> Self {
        Point2D::from(self.x - b.x, self.y - b.y)
    }
}

impl ops::Sub<&Point2D> for Point2D {
    type Output = Self;
    
    fn sub(self, b: &Self) -> Self {
        Point2D::from(self.x - b.x, self.y - b.y)
    }
}

impl ops::Mul for Point2D {
    type Output = Self;
    
    fn mul(self, b: Self) -> Self {
        Point2D::from(self.x * b.x, self.y * b.y)
    }
}

impl ops::Mul<&Point2D> for Point2D {
    type Output = Self;
    
    fn mul(self, b: &Self) -> Self {
        Point2D::from(self.x * b.x, self.y * b.y)
    }
}


impl ops::Mul<f64> for Point2D {
    type Output = Self;

    fn mul(self, b: f64) -> Self {
        Point2D::from(self.x * b, self.y * b)
    }
}

impl ops::Mul<&f64> for Point2D {
    type Output = Self;

    fn mul(self, b: &f64) -> Self {
        Point2D::from(self.x * b, self.y * b)
    }
}

impl PartialEq for Point2D {
    fn eq(&self, b: &Self) -> bool {
        self.dist_approx(*b) < EQUAL_THRESHOLD
    }
}
