use nalgebra::Vector3;

pub trait Geometry: BoundingBox {}

pub trait BoundingBox {
    /// Compute the minimum coordinate of the bounding box.
    fn bb_min(&self) -> Vector3<f32>;
    /// Compute the maximum coordinate of the bounding box.
    fn bb_max(&self) -> Vector3<f32>;
    /// Compute the minimum and maximum coordinate of the bounding box.
    fn bb_min_max(&self) -> (Vector3<f32>, Vector3<f32>) {
        (self.bb_min(), self.bb_max())
    }
}
