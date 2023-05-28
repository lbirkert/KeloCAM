use std::ops;

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

}

impl ops::Add for Point2D {
    type Output = Self;
    
    fn add(self, b: Self) -> Self {
        Point2D::from(self.x + b.x, self.y + b.y)
    }
}

impl ops::Sub for Point2D {
    type Output = Self;
    
    fn sub(self, b: Self) -> Self {
        Point2D::from(self.x - b.x, self.y - b.y)
    }
}

impl ops::Mul for Point2D {
    type Output = Self;
    
    fn mul(self, b: Self) -> Self {
        Point2D::from(self.x * b.x, self.y * b.y)
    }
}

impl ops::Mul<f64> for Point2D {
    type Output = Self;

    fn mul(self, b: f64) -> Self {
        Point2D::from(self.x * b, self.y * b)
    }

}

impl PartialEq for Point2D {
    fn eq(&self, b: &Self) -> bool {
        self.x == b.x && self.y == b.y
    }
}
