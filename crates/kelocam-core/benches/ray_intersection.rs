#![feature(test)]

extern crate test;
use kelocam_core::primitives::{Plane, Ray, Square, Triangle};
use nalgebra::{UnitVector3, Vector3};
use test::Bencher;

#[bench]
pub fn ray_triangle_intersection(bencher: &mut Bencher) {
    bencher.iter(|| {
        let triangle: Triangle;
        {
            let a = Vector3::new_random();
            let b = Vector3::new_random();
            let c = Vector3::new_random();
            let normal = UnitVector3::new_normalize((a - b).cross(&(a - c)));
            triangle = Triangle::new(a, b, c, normal);
        }

        let ray: Ray;
        {
            let origin = Vector3::new_random();
            let normal = UnitVector3::new_normalize(Vector3::new_random());
            ray = Ray::new(origin, normal);
        }

        ray.intersect(&triangle)
    })
}

#[bench]
pub fn ray_plane_intersection(bencher: &mut Bencher) {
    bencher.iter(|| {
        let plane: Plane;
        {
            let origin = Vector3::new_random();
            let normal = UnitVector3::new_normalize(Vector3::new_random());
            plane = Plane::new(origin, normal);
        }

        let ray: Ray;
        {
            let origin = Vector3::new_random();
            let normal = UnitVector3::new_normalize(Vector3::new_random());
            ray = Ray::new(origin, normal);
        }

        ray.intersect(&plane)
    })
}

#[bench]
pub fn ray_square_intersection(bencher: &mut Bencher) {
    bencher.iter(|| {
        let square: Square;
        {
            let a = Vector3::new_random();
            let ab = Vector3::new_random();
            let ac = Vector3::new_random();
            let normal = UnitVector3::new_normalize(ab.cross(&ac));
            square = Square::new(a, ab, ac, normal);
        }

        let ray: Ray;
        {
            let origin = Vector3::new_random();
            let normal = UnitVector3::new_normalize(Vector3::new_random());
            ray = Ray::new(origin, normal);
        }

        ray.intersect(&square)
    })
}
