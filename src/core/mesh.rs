use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
};

use nalgebra::{Matrix4, UnitVector3, Vector3};

use crate::core::Line;

use super::{Path3, Plane, Ray, Triangle};

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
    pub fn intersect_ray(&self, ray: &Ray) -> Option<Vector3<f32>> {
        Self::intersect_ray_raw(&self.triangles, ray)
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
    pub fn slice(&self, plane: Plane) -> Vec<Path3> {
        Self::slice_raw(&self.triangles, plane)
    }

    /// Slice a model using a plane. This returns the outline of the cross section.
    pub fn slice_raw(triangles: &[Triangle], plane: Plane) -> Vec<Path3> {
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
            let pa = Line::intersect_plane_raw(&a, &b, &plane);
            let pb = Line::intersect_plane_raw(&b, &c, &plane);
            let pc = Line::intersect_plane_raw(&c, &a, &plane);

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

    /// Extrude this mesh outwards by factor. This is used for cutter compensation.
    pub fn extrude_xy(&self, factor: f32) -> Mesh {
        let mut triangles: Vec<Triangle> = Vec::new();

        const EPSILON: f32 = 1e-9;

        let mut points: Vec<Vector3<f32>> = Vec::new();
        let mut edges: HashMap<(usize, usize), Vec<(usize, Edge)>> = HashMap::new();
        let mut corners: Vec<Vec<(usize, Vertex)>> = Vec::new();

        // Construct edges
        for (i, triangle) in self.triangles.iter().enumerate() {
            let ai = 'ai: {
                for (i, point) in points.iter().enumerate() {
                    if (triangle.a - point).magnitude_squared() < EPSILON {
                        break 'ai i;
                    }
                }
                points.push(triangle.a);
                corners.push(Vec::new());
                points.len() - 1
            };
            let bi = 'bi: {
                for (i, point) in points.iter().enumerate() {
                    if (triangle.b - point).magnitude_squared() < EPSILON {
                        break 'bi i;
                    }
                }
                points.push(triangle.b);
                corners.push(Vec::new());
                points.len() - 1
            };
            let ci = 'ci: {
                for (i, point) in points.iter().enumerate() {
                    if (triangle.c - point).magnitude_squared() < EPSILON {
                        break 'ci i;
                    }
                }
                points.push(triangle.c);
                corners.push(Vec::new());
                points.len() - 1
            };

            corners[ai].push((i, Vertex::A));
            corners[bi].push((i, Vertex::B));
            corners[ci].push((i, Vertex::C));

            let e1 = (ai.min(bi), ai.max(bi));
            let e2 = (bi.min(ci), bi.max(ci));
            let e3 = (ci.min(ai), ci.max(ai));

            if let Some(ref mut edge) = edges.get_mut(&e1) {
                edge.push((i, Edge::A));
            } else {
                edges.insert(e1, vec![(i, Edge::A)]);
            }

            if let Some(ref mut edge) = edges.get_mut(&e2) {
                edge.push((i, Edge::B));
            } else {
                edges.insert(e2, vec![(i, Edge::B)]);
            }

            if let Some(ref mut edge) = edges.get_mut(&e3) {
                edge.push((i, Edge::C));
            } else {
                edges.insert(e3, vec![(i, Edge::C)]);
            }
        }

        for triangle in self.triangles.iter() {
            let mut triangle = triangle.clone();

            let mut e = Vector3::new(triangle.normal.x, triangle.normal.y, 0.0);
            //let mut e = triangle.normal.into_inner();
            if e.magnitude_squared() > 0.0 {
                e.normalize_mut();
                e.scale_mut(factor);
            }
            triangle.a += e;
            triangle.b += e;
            triangle.c += e;
            triangles.push(triangle);
        }

        for (_, edge) in edges.iter_mut() {
            let (ta, tb) = match (edge.pop(), edge.pop()) {
                (Some(ta), Some(tb)) => (ta, tb),
                _ => continue,
            };

            let na = triangles[ta.0].normal;
            let nb = triangles[tb.0].normal;

            //let (&a0, &a1) = ta.1.get(&triangles[ta.0]);
            //let (&b0, &b1) = ta.1.get(&triangles[ta.0]);

            //// No transformation
            //if (c0 - a0).magnitude_squared() < EPSILON || (c0 - b0).magnitude_squared() < EPSILON {
            //    let normal = UnitVector3::new_normalize(na.into_inner() + nb.into_inner());
            //    triangles.push(Triangle::new(a1, b1, a0, normal));
            //    triangles.push(Triangle::new(a1, b0, b1, normal));
            //    continue;
            //}

            // Center for radial extrusion
            let tra = &self.triangles[ta.0];
            let (c0, c1) = ta.1.get(tra);

            {
                let center = (tra.a + tra.b + tra.c).scale(1.0 / 3.0);
                if nb.dot(&(center - c0)) * factor.signum() > 0.0 {
                    continue;
                }
            }

            let mut nna = Vector3::new(na.x, na.y, 0.0);
            let mut nnb = Vector3::new(nb.x, nb.y, 0.0);
            nna.normalize_mut();
            nnb.normalize_mut();
            let mut delta = nna.dot(&nnb).acos();
            if delta > 0.0 && delta < std::f32::consts::PI {
                let yfac = nna;
                let mut xfac = nnb.cross(&nna).cross(&nna);
                xfac.normalize_mut();

                let res: i32 = ((delta / std::f32::consts::PI) * 10.0 * factor).ceil() as i32;

                delta *= xfac.dot(&nb).signum();

                let mut b = yfac.scale(factor);
                for i in 1..=res {
                    let normal =
                        UnitVector3::new_normalize(na.lerp(&nb, i as f32 / (res + 1) as f32));
                    let angle = delta * (i as f32 / res as f32);
                    let (mut sin, mut cos) = angle.sin_cos();
                    sin *= factor;
                    cos *= factor;

                    let a = xfac.scale(sin) + yfac.scale(cos);
                    let a0 = c0 + a;
                    let a1 = c1 + a;
                    let b0 = c0 + b;
                    let b1 = c1 + b;

                    triangles.push(Triangle::new(a1, b0, a0, normal));
                    triangles.push(Triangle::new(a1, b1, b0, normal));

                    b = a;
                }
            } else {
                let (&a0, &a1) = ta.1.get(&triangles[ta.0]);
                let (&b0, &b1) = tb.1.get(&triangles[tb.0]);
                let mut normal = Vector3::z_axis();
                if (na.into_inner() + nb.into_inner()).dot(&normal) < 0.0 {
                    normal = -normal;
                }

                //let normal = UnitVector3::new_normalize(na.into_inner() + nb.into_inner());
                triangles.push(Triangle::new(a1, b1, a0, normal));
                triangles.push(Triangle::new(a1, b0, b1, normal));
            }
        }

        /*
            for corner in corners.iter() {
                // point + normal
                let mut points: Vec<(Vector3<f32>, Vector3<f32>)> = Vec::new();

                for point in corner.iter() {
                    let triangle = &triangles[point.0];
                    let &point = point.1.get(triangle);
                    let different = !points
                        .iter()
                        .any(|other| (point - other.0).magnitude_squared() < EPSILON);

                    if different {
                        points.push((point, triangle.normal.into_inner()));
                    }
                }

                loop {
                    let (a, b, c) = match (points.pop(), points.pop(), points.pop()) {
                        (Some(a), Some(b), Some(c)) => (a, b, c),
                        _ => break,
                    };

                    triangles.push(Triangle::new(
                        a.0,
                        b.0,
                        c.0,
                        UnitVector3::new_normalize(a.1 + b.1 + c.1),
                    ));
                }
            }
        */

        Mesh::new(triangles)
    }
}

// TODO: move to triangle.rs
#[derive(Debug)]
pub enum Vertex {
    A,
    B,
    C,
}

impl Vertex {
    #[inline]
    pub fn get<'a>(&self, triangle: &'a Triangle) -> &'a Vector3<f32> {
        match self {
            Self::A => &triangle.a,
            Self::B => &triangle.b,
            Self::C => &triangle.c,
        }
    }
}

#[derive(Debug)]
pub enum Edge {
    A,
    B,
    C,
}

impl Edge {
    #[inline]
    pub fn get<'a>(&self, triangle: &'a Triangle) -> (&'a Vector3<f32>, &'a Vector3<f32>) {
        match self {
            Self::A => (&triangle.a, &triangle.b),
            Self::B => (&triangle.b, &triangle.c),
            Self::C => (&triangle.c, &triangle.a),
        }
    }
}
