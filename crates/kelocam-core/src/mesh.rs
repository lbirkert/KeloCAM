use std::{collections::HashSet, io::Cursor};

use nalgebra::{Matrix4, UnitVector3, Vector3};

use crate::{ray::RayIntersection, BoundingBox, Geometry};

use super::{Line, Path3, Plane, Ray, Triangle};

#[derive(Debug, Clone)]
pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        Self { triangles }
    }

    /// Read mesh from stl file.
    pub fn from_stl(cursor: &mut Cursor<&[u8]>) -> std::io::Result<Mesh> {
        stl::read_stl(cursor)
            .map(|stl| Self::new(stl.triangles.into_iter().map(Triangle::from_stl).collect()))
    }

    /// Perform a ray intersection with this mesh.
    /// Returns the intersection point if any, otherwise None.
    pub fn intersect_ray_raw(triangles: &[Triangle], ray: &Ray) -> Option<Vector3<f32>> {
        let mut intersection_dist = std::f32::INFINITY;
        let mut intersection_point = None;
        for triangle in triangles.iter() {
            if let Some(point) = triangle.intersect_ray(ray) {
                let dist = (point - ray.origin).magnitude_squared();

                if dist < intersection_dist {
                    intersection_dist = dist;
                    intersection_point = Some(point);
                }
            }
        }
        intersection_point
    }

    /// Slice a model using a plane. This returns the outline of the cross section.
    pub fn slice(&self, plane: &Plane) -> Vec<Path3> {
        Self::slice_raw(&self.triangles, plane)
    }

    /// Slice a model using a plane. This returns the outline of the cross section.
    pub fn slice_raw(triangles: &[Triangle], plane: &Plane) -> Vec<Path3> {
        const EPSILON: f32 = 1e-9;

        // Used for connecting the polygons properly. TODO: Look into hashing
        let mut points: Vec<Vector3<f32>> = Vec::new();
        let mut indicies: Vec<Vec<usize>> = Vec::new();

        // Used for sorting out duplicates
        let mut segments: HashSet<(usize, usize)> = HashSet::new();

        for triangle in triangles.iter() {
            let a = triangle.a;
            let b = triangle.b;
            let c = triangle.c;
            let pa = Line::intersect_plane_raw(&a, &b, plane);
            let pb = Line::intersect_plane_raw(&b, &c, plane);
            let pc = Line::intersect_plane_raw(&c, &a, plane);

            let seg = match (pa, pb, pc) {
                (Some(pa), Some(pb), _) => (pa, pb),
                (_, Some(pb), Some(pc)) => (pb, pc),
                (Some(pa), _, Some(pc)) => (pc, pa),
                _ => continue,
            };

            // Skip 'zero' length segments
            if (seg.0 - seg.1).magnitude_squared() < EPSILON {
                continue;
            }

            let delta = plane.normal.cross(&triangle.normal);
            let (a, b) = if (seg.1 - seg.0).dot(&delta) > 0.0 {
                (seg.0, seg.1)
            } else {
                (seg.1, seg.0)
            };

            let ai = 'ai: {
                for (i, point) in points.iter().enumerate() {
                    if (a - point).magnitude_squared() < EPSILON {
                        break 'ai i;
                    }
                }
                points.push(a);
                indicies.push(Vec::new());
                points.len() - 1
            };

            let bi = 'bi: {
                for (i, point) in points.iter().enumerate() {
                    if (b - point).magnitude_squared() < EPSILON {
                        break 'bi i;
                    }
                }
                points.push(b);
                indicies.push(Vec::new());
                points.len() - 1
            };

            if segments.insert((ai.min(bi), ai.max(bi))) {
                indicies[ai].push(bi);
            }
        }

        if indicies.is_empty() {
            return Vec::new();
        }

        let mut paths = Vec::new();
        let mut path = Vec::new();

        let mut pointer = 0;
        loop {
            if indicies[pointer].is_empty() {
                paths.push(Path3::new_sanitize(path));

                let mut found = None;
                for (i, index) in indicies.iter().enumerate() {
                    if !index.is_empty() {
                        found = Some(i);
                        break;
                    }
                }

                if let Some(found) = found {
                    path = Vec::new();
                    pointer = found;
                } else {
                    break;
                }
            } else {
                path.push(points[pointer]);
                pointer = indicies[pointer].pop().unwrap();
            }
        }

        paths
    }

    /// Translate (aka. move) this mesh by the specified amount in delta. Internally uses `point +=
    /// delta` on every vertex to perform the transformation. The normals are unaffected by this transformation.
    pub fn translate(&mut self, delta: &Vector3<f32>) {
        for triangle in self.triangles.iter_mut() {
            triangle.a += delta;
            triangle.b += delta;
            triangle.c += delta;
        }
    }

    /// Scale this mesh non uniformly by the specified amount in delta. Internally uses
    /// `point.component_mul_assign(delta)` on every vertex. To perform the transformation.
    /// This will also be applied to normals, which will then be renormalized after the
    /// transformation. Use scale if you do not require non uniform scaling, which does
    /// not have to renormalize the normal vectors.
    pub fn scale_non_uniformly(&mut self, delta: &Vector3<f32>) {
        for triangle in self.triangles.iter_mut() {
            triangle.a.component_mul_assign(delta);
            triangle.b.component_mul_assign(delta);
            triangle.c.component_mul_assign(delta);
            triangle.normal = UnitVector3::new_normalize(triangle.normal.component_mul(delta));
        }
    }

    /// Scale this mesh uniformly by the specified amount in delta. Internally uses `point.scale_mut(delta)`
    /// on every vertex to perform the transformation. If the delta factor is negative, the
    /// triangle normal will get inverted as well.
    pub fn scale(&mut self, delta: f32) {
        let normal = delta.signum();
        for triangle in self.triangles.iter_mut() {
            triangle.a.scale_mut(delta);
            triangle.b.scale_mut(delta);
            triangle.c.scale_mut(delta);
            triangle.normal = UnitVector3::new_unchecked(triangle.normal.scale(normal));
        }
    }

    /// Rotate this mesh by the specified euler angles in delta. Internally uses a rotation matrix
    /// on every vertex to perform this transformation. This will be applied to the normal vectors
    /// as well.
    pub fn rotate(&mut self, delta: &Vector3<f32>) {
        let mat = Matrix4::from_euler_angles(delta.x, delta.y, delta.z);
        for triangle in self.triangles.iter_mut() {
            triangle.a = mat.transform_vector(&triangle.a);
            triangle.b = mat.transform_vector(&triangle.b);
            triangle.c = mat.transform_vector(&triangle.c);
            triangle.normal = UnitVector3::new_unchecked(mat.transform_vector(&triangle.normal));
        }
    }
}

impl RayIntersection for Mesh {
    fn intersect_ray(&self, ray: &Ray) -> Option<Vector3<f32>> {
        Self::intersect_ray_raw(&self.triangles, ray)
    }
}

impl BoundingBox for Mesh {
    fn bb_min(&self) -> Vector3<f32> {
        let mut min = Vector3::from_element(std::f32::INFINITY);
        for triangle in self.triangles.iter() {
            min = min.inf(&triangle.a.inf(&triangle.b.inf(&triangle.c)));
        }
        min
    }

    fn bb_max(&self) -> Vector3<f32> {
        let mut max = Vector3::from_element(std::f32::NEG_INFINITY);
        for triangle in self.triangles.iter() {
            max = max.sup(&triangle.a.sup(&triangle.b.sup(&triangle.c)));
        }
        max
    }
}

impl Geometry for Mesh {}
