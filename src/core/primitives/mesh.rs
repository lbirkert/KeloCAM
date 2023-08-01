use std::{collections::HashSet, io::Cursor};

use nalgebra::{Matrix4, UnitVector3, Vector2, Vector3};

use super::{Path2, Ray, Triangle};

#[derive(Clone)]
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
    pub fn intersection(&self, ray: &Ray) -> Option<Vector3<f32>> {
        Self::intersect_raw(&self.triangles, ray)
    }

    /// Perform a ray intersection with this mesh.
    /// Returns the intersection point if any, otherwise None.
    pub fn intersect_raw(triangles: &[Triangle], ray: &Ray) -> Option<Vector3<f32>> {
        let mut intersection_dist = std::f32::INFINITY;
        let mut intersection_point = None;
        for triangle in triangles.iter() {
            if let Some(point) = triangle.intersect(ray) {
                let dist = (point - ray.origin).magnitude_squared();

                if dist < intersection_dist {
                    intersection_dist = dist;
                    intersection_point = Some(point);
                }
            }
        }
        intersection_point
    }

    /// Z-Slice a model. Creates a path representing the outline of this Mesh at this z height.
    pub fn z_slice(&self, z: f32) -> Vec<Path2> {
        Self::z_slice_raw(&self.triangles, z)
    }

    /// Z-Slice a model. Creates a path representing the outline of this Mesh at this z height.
    pub fn z_slice_raw(triangles: &[Triangle], z: f32) -> Vec<Path2> {
        // Used for connecting the polygons properly
        let mut points: Vec<Vector2<f32>> = Vec::new();
        let mut segments: HashSet<(usize, usize)> = HashSet::new();
        let mut indicies: Vec<(usize, bool)> = Vec::new();

        for triangle in triangles.iter() {
            let a = triangle.a;
            let b = triangle.b;
            let c = triangle.c;
            let pa = if (a.z > z) != (b.z > z) {
                Some(a.xy().lerp(&b.xy(), 1.0 - (z - b.z) / (a.z - b.z)))
            } else {
                None
            };
            let pb = if (b.z > z) != (c.z > z) {
                Some(b.xy().lerp(&c.xy(), 1.0 - (z - c.z) / (b.z - c.z)))
            } else {
                None
            };
            let pc = if (c.z > z) != (a.z > z) {
                Some(c.xy().lerp(&a.xy(), 1.0 - (z - a.z) / (c.z - a.z)))
            } else {
                None
            };

            let segment = match (pa, pb, pc) {
                (Some(pa), Some(pb), _) => (pa, pb),
                (_, Some(pb), Some(pc)) => (pb, pc),
                (Some(pa), _, Some(pc)) => (pc, pa),
                _ => continue,
            };

            const EPSILON: f32 = 1e-9;

            // Skip 'zero' length segments
            if (segment.0 - segment.1).magnitude_squared() < EPSILON {
                continue;
            }

            let a: Vector2<f32>;
            let b: Vector2<f32>;

            let delta = Vector3::z_axis().cross(&triangle.normal).xy();
            if (segment.1 - segment.0).dot(&delta) > 0.0 {
                a = segment.0;
                b = segment.1;
            } else {
                b = segment.0;
                a = segment.1;
            }

            let mut ai = None;
            for (i, point) in points.iter().enumerate() {
                if (a - point).magnitude_squared() < EPSILON {
                    ai = Some(i);
                    break;
                }
            }

            let ai = ai.unwrap_or_else(|| {
                points.push(a);
                indicies.push((0, false));
                points.len() - 1
            });

            let mut bi = None;
            for (i, point) in points.iter().enumerate() {
                if (b - point).magnitude_squared() < EPSILON {
                    bi = Some(i);
                    break;
                }
            }

            let bi = bi.unwrap_or_else(|| {
                points.push(b);
                indicies.push((0, false));
                points.len() - 1
            });

            // Check for zero area path
            if !segments.remove(&(bi, ai)) {
                segments.insert((ai, bi));
            }
        }

        for segment in segments {
            indicies[segment.0].0 = segment.1;
        }

        if indicies.is_empty() {
            return Vec::new();
        }

        let mut paths = Vec::new();
        let mut path = Vec::new();

        let mut pointer = 0;
        loop {
            if indicies[pointer].1 {
                paths.push(Path2::new(path));

                let mut found = None;
                for (i, index) in indicies.iter().enumerate() {
                    if !index.1 {
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
                indicies[pointer].1 = true;
                pointer = indicies[pointer].0;
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

    /// Calculate the infinum (aka. componentwise min) and the suprenum (aka. componentwise max) of this mesh.
    pub fn inf_sup(&self) -> (Vector3<f32>, Vector3<f32>) {
        let mut inf = Vector3::from_element(std::f32::INFINITY);
        let mut sup = Vector3::from_element(std::f32::NEG_INFINITY);

        for triangle in self.triangles.iter() {
            inf = inf.inf(&triangle.a.inf(&triangle.b.inf(&triangle.c)));
            sup = sup.sup(&triangle.a.sup(&triangle.b.sup(&triangle.c)));
        }

        (inf, sup)
    }
}
