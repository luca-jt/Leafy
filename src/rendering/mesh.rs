use crate::ecs::component::Scale;
use crate::glm;
use crate::utils::constants::ORIGIN;
use crate::utils::tools::to_vec4;
use gl::types::*;
use obj::{load_obj, Obj, TexturedVertex};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// vertex used in the AOS meshes
#[derive(Debug, Copy, Clone)]
struct MeshVertex {
    position: glm::Vec3,
    uv: glm::Vec2,
    normal: glm::Vec3,
}

/// mesh containing vertex structs as opposed to the regular soa mesh which allows for easier processing
struct AOSMesh {
    vertices: Vec<MeshVertex>,
    faces: Vec<[usize; 3]>,
}

impl AOSMesh {
    /// converts back to a regular mesh
    fn to_mesh(self) -> Mesh {
        Mesh {
            positions: self.vertices.iter().map(|v| v.position).collect(),
            texture_coords: self.vertices.iter().map(|v| v.uv).collect(),
            normals: self.vertices.iter().map(|v| v.normal).collect(),
            indices: self
                .faces
                .into_iter()
                .flatten()
                .map(|i| i as GLuint)
                .collect(),
            max_reach: self.vertices.iter().map(|v| v.position.abs()).fold(
                ORIGIN,
                |mut current, p| {
                    current.x = current.x.max(p.x);
                    current.y = current.y.max(p.y);
                    current.z = current.z.max(p.z);
                    current
                },
            ),
        }
    }

    /// creates a hitbox in the form of a unaltered version of the mesh
    fn hitbox_mesh(self) -> HitboxMesh {
        HitboxMesh {
            vertices: self.vertices.into_iter().map(|v| v.position).collect(),
            faces: self.faces,
        }
    }

    /// creates a hitbox in the form of a simplified version of the mesh
    fn simplified(mut self) -> Self {
        let target_triangle_count = (self.faces.len() / 2).max(4);
        let mut vertex_errors: Vec<glm::Mat4> = self.calculate_vertex_errors();
        let mut edge_queue = self.build_edge_queue(&vertex_errors);
        while self.faces.len() > target_triangle_count && !edge_queue.is_empty() {
            let edge = edge_queue.pop().unwrap();
            self.collapse_edge(edge, &mut vertex_errors);
        }
        self.remove_unused_vertices();
        self
    }

    /// removes vertices that are not used by a face and corrects the face indices
    fn remove_unused_vertices(&mut self) {
        let mut used_indices = self.faces.iter().flatten().copied().collect::<HashSet<_>>();
        for i in 0..self.vertices.len() {
            while !used_indices.contains(&i) {
                self.vertices.swap_remove(i);
                self.faces
                    .iter_mut()
                    .flatten()
                    .filter(|index| **index == self.vertices.len())
                    .for_each(|index| *index = i);
                if used_indices.remove(&self.vertices.len()) {
                    used_indices.insert(i);
                }
            }
            if i == self.vertices.len() {
                break;
            }
        }
    }

    /// calculate the initial error metrics (quadrics) for all vertices
    fn calculate_vertex_errors(&self) -> Vec<glm::Mat4> {
        let mut vertex_errors = vec![glm::Mat4::zeros(); self.vertices.len()];
        for face in self.faces.iter() {
            let (v0, v1, v2) = (
                self.vertices[face[0]].position,
                self.vertices[face[1]].position,
                self.vertices[face[2]].position,
            );
            let normal = (v1 - v0).cross(&(v2 - v0)).normalize();
            let d = -normal.dot(&v0);

            let quadric = glm::mat4(
                normal.x * normal.x,
                normal.x * normal.y,
                normal.x * normal.z,
                normal.x * d,
                normal.y * normal.x,
                normal.y * normal.y,
                normal.y * normal.z,
                normal.y * d,
                normal.z * normal.x,
                normal.z * normal.y,
                normal.z * normal.z,
                normal.z * d,
                d * normal.x,
                d * normal.y,
                d * normal.z,
                d * d,
            );
            for vi in face {
                vertex_errors[*vi] += quadric;
            }
        }
        vertex_errors
    }

    /// build a min heap of edges prioritized by error metrics
    fn build_edge_queue(&self, vertex_errors: &[glm::Mat4]) -> BinaryHeap<Edge> {
        let mut edge_queue = BinaryHeap::new();
        let mut edge_set = HashMap::new();
        for face in &self.faces {
            for i in 0..3 {
                let (v1, v2) = (face[i], face[(i + 1) % 3]);
                if !edge_set.contains_key(&(v1.min(v2), v1.max(v2))) {
                    let error = self.calculate_edge_error(v1, v2, vertex_errors);
                    edge_queue.push(Edge { v1, v2, error });
                    edge_set.insert((v1.min(v2), v1.max(v2)), true);
                }
            }
        }
        edge_queue
    }

    /// calculate the error for collapsing an edge between vertices v1 and v2
    fn calculate_edge_error(&self, v1: usize, v2: usize, vertex_errors: &[glm::Mat4]) -> f32 {
        let quadric = vertex_errors[v1] + vertex_errors[v2];
        let midpoint = (self.vertices[v1].position + self.vertices[v2].position) * 0.5;
        let midpoint_hom = glm::vec4(midpoint.x, midpoint.y, midpoint.z, 1.0);
        (midpoint_hom.transpose() * quadric * midpoint_hom).sum()
    }

    /// collapse an edge and update the mesh structure
    fn collapse_edge(&mut self, edge: Edge, vertex_errors: &mut [glm::Mat4]) {
        let Edge { v1, v2, .. } = edge;
        self.vertices[v1] = MeshVertex {
            position: (self.vertices[v1].position + self.vertices[v2].position) * 0.5,
            uv: (self.vertices[v1].uv + self.vertices[v2].uv) * 0.5,
            normal: (self.vertices[v1].normal + self.vertices[v2].normal) * 0.5,
        };
        vertex_errors[v1] += vertex_errors[v2];
        self.faces.retain_mut(|face| {
            for v in face.iter_mut() {
                if *v == v2 {
                    *v = v1;
                }
            }
            face[0] != face[1] && face[1] != face[2] && face[2] != face[0]
        });
    }
}

/// a mesh that can be rendered in gl
pub(crate) struct Mesh {
    pub(crate) positions: Vec<glm::Vec3>,
    pub(crate) texture_coords: Vec<glm::Vec2>,
    pub(crate) normals: Vec<glm::Vec3>,
    pub(crate) indices: Vec<GLuint>,
    pub(crate) max_reach: glm::Vec3,
}

impl Mesh {
    /// creates a new Mesh from an obj file path
    pub(crate) fn from_path(file_path: impl AsRef<Path>) -> Self {
        let data = BufReader::new(File::open(file_path).expect("file not found"));
        let model: Obj<TexturedVertex, GLuint> = load_obj(data).unwrap();
        Self::from_obj(model)
    }

    /// creates a new Mesh from a byte array
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        let data = BufReader::new(bytes);
        let model: Obj<TexturedVertex, GLuint> = load_obj(data).unwrap();
        Self::from_obj(model)
    }

    /// converts the obj file format into the required data structures
    fn from_obj(obj: Obj<TexturedVertex, GLuint>) -> Self {
        // convert the data into the required format
        let positions: Vec<glm::Vec3> = obj
            .vertices
            .iter()
            .map(|vertex| glm::vec3(vertex.position[0], vertex.position[1], vertex.position[2]))
            .collect();

        let texture_coords: Vec<glm::Vec2> = obj
            .vertices
            .iter()
            .map(|vertex| glm::vec2(vertex.texture[0], vertex.texture[1]))
            .collect();

        let normals: Vec<glm::Vec3> = obj
            .vertices
            .iter()
            .map(|vertex| glm::vec3(vertex.normal[0], vertex.normal[1], vertex.normal[2]))
            .collect();

        let indices: Vec<GLuint> = obj.indices;
        assert_eq!(indices.len() % 3, 0, "mesh has to be triangulated");

        let max_reach =
            positions
                .iter()
                .copied()
                .map(|p| p.abs())
                .fold(ORIGIN, |mut current, p| {
                    current.x = current.x.max(p.x);
                    current.y = current.y.max(p.y);
                    current.z = current.z.max(p.z);
                    current
                });

        Self {
            positions,
            texture_coords,
            normals,
            indices,
            max_reach,
        }
    }

    /// yields the number of vertices per object
    #[inline]
    pub(crate) fn num_vertices(&self) -> usize {
        self.positions.len()
    }

    /// yields the number of indices per object
    #[inline]
    pub(crate) fn num_indices(&self) -> usize {
        self.indices.len()
    }

    /// generates the AOS mesh for easier data parsing
    fn aos_mesh(&self) -> AOSMesh {
        let mut faces = vec![[0, 0, 0]; self.indices.len() / 3];
        for (i, index) in self.indices.iter().enumerate() {
            faces[i / 3][i % 3] = *index as usize;
        }
        AOSMesh {
            vertices: self
                .positions
                .iter()
                .zip(self.texture_coords.iter())
                .zip(self.normals.iter())
                .map(|data| MeshVertex {
                    position: *data.0 .0,
                    uv: *data.0 .1,
                    normal: *data.1,
                })
                .collect(),
            faces,
        }
    }

    /// generates the inverse inertia tensor matrix, center of mass and the mass
    pub(crate) fn intertia_data(&self, density: f32, scale: &Scale) -> (glm::Mat3, glm::Vec3, f32) {
        // inertia matrix entries
        let (mut ia, mut ib, mut ic, mut iap, mut ibp, mut icp) =
            (0f32, 0f32, 0f32, 0f32, 0f32, 0f32);
        let mut center_of_mass = ORIGIN;
        let mut mass = 0f32;
        let scale_matrix = scale.scale_matrix();

        for i in (0..self.indices.len()).step_by(3) {
            let pos1 = self.positions[self.indices[i] as usize];
            let pos2 = self.positions[self.indices[i + 1] as usize];
            let pos3 = self.positions[self.indices[i + 2] as usize];
            let scaled1 = (scale_matrix * to_vec4(&pos1)).xyz();
            let scaled2 = (scale_matrix * to_vec4(&pos2)).xyz();
            let scaled3 = (scale_matrix * to_vec4(&pos3)).xyz();
            let triangle = (scaled1, scaled2, scaled3);
            let det_jacobi = triangle.0.dot(&triangle.1.cross(&triangle.2));
            let tet_volume = det_jacobi / 6.0;
            let tet_mass = tet_volume * density;
            let tet_mass_center = (triangle.0 + triangle.1 + triangle.2) / 4.0;

            ia += det_jacobi * (inertia_moment(&triangle, 1) + inertia_moment(&triangle, 2));
            ib += det_jacobi * (inertia_moment(&triangle, 0) + inertia_moment(&triangle, 2));
            ic += det_jacobi * (inertia_moment(&triangle, 0) + inertia_moment(&triangle, 1));
            iap += det_jacobi * inertia_product(&triangle, 1, 2);
            ibp += det_jacobi * inertia_product(&triangle, 0, 1);
            icp += det_jacobi * inertia_product(&triangle, 0, 2);

            center_of_mass += tet_mass * tet_mass_center;
            mass += tet_mass;
        }
        center_of_mass /= mass;
        ia = density * ia / 60.0
            - mass * (center_of_mass.y * center_of_mass.y + center_of_mass.z * center_of_mass.z);
        ib = density * ib / 60.0
            - mass * (center_of_mass.x * center_of_mass.x + center_of_mass.z * center_of_mass.z);
        ic = density * ic / 60.0
            - mass * (center_of_mass.x * center_of_mass.x + center_of_mass.y * center_of_mass.y);
        iap = density * ia / 120.0 - mass * center_of_mass.y * center_of_mass.z;
        ibp = density * ia / 120.0 - mass * center_of_mass.x * center_of_mass.y;
        icp = density * ia / 120.0 - mass * center_of_mass.x * center_of_mass.z;

        (
            glm::Mat3::from_columns(&[
                glm::vec3(ia, -ibp, -icp),
                glm::vec3(-ibp, ib, -iap),
                glm::vec3(-icp, -iap, ic),
            ])
            .try_inverse()
            .unwrap(),
            center_of_mass,
            mass,
        )
    }

    /// generates all the simpified meshes for the lod levels
    #[rustfmt::skip]
    pub(crate) fn generate_lods(&self) -> [Mesh; 4] {
        [
            self.aos_mesh().simplified().to_mesh(),
            self.aos_mesh().simplified().simplified().to_mesh(),
            self.aos_mesh().simplified().simplified().simplified().to_mesh(),
            self.aos_mesh().simplified().simplified().simplified().simplified().to_mesh(),
        ]
    }

    /// generates the meshes' hitbox for the given hitbox type
    #[rustfmt::skip]
    pub(crate) fn generate_hitbox(&self, hitbox: &HitboxType) -> Hitbox {
        match hitbox {
            HitboxType::ConvexHull => Hitbox::ConvexMesh(self.aos_mesh().hitbox_mesh().convex_hull()),
            HitboxType::SimplifiedConvexHull => Hitbox::ConvexMesh(self.aos_mesh().simplified().hitbox_mesh().convex_hull()),
            HitboxType::Ellipsiod => Hitbox::Ellipsoid(self.max_reach),
            HitboxType::Box => Hitbox::Box(HitboxMesh::box_from_dimensions(&self.max_reach)),
        }
    }
}

/// computes the inertia moment for a given traingle and index
fn inertia_moment(triangle: &(glm::Vec3, glm::Vec3, glm::Vec3), i: usize) -> f32 {
    triangle.0[i] * triangle.0[i]
        + triangle.1[i] * triangle.2[i]
        + triangle.1[i] * triangle.1[i]
        + triangle.0[i] * triangle.2[i]
        + triangle.2[i] * triangle.2[i]
        + triangle.0[i] * triangle.1[i]
}

/// computes the inertia product for a given traingle and indices
fn inertia_product(triangle: &(glm::Vec3, glm::Vec3, glm::Vec3), i: usize, j: usize) -> f32 {
    2.0 * triangle.0[i] * triangle.0[j]
        + triangle.1[i] * triangle.2[j]
        + triangle.2[i] * triangle.1[j]
        + 2.0 * triangle.1[i] * triangle.1[j]
        + triangle.0[i] * triangle.2[j]
        + triangle.2[i] * triangle.0[j]
        + 2.0 * triangle.2[i] * triangle.2[j]
        + triangle.0[i] * triangle.1[j]
        + triangle.1[i] * triangle.0[j]
}

/// hitbox type specifier for an entity (enables collision physics, requires MeshType to work)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HitboxType {
    ConvexHull,
    SimplifiedConvexHull,
    Ellipsiod,
    Box,
}

/// all possible versions of hitboxes
pub(crate) enum Hitbox {
    ConvexMesh(HitboxMesh),
    Ellipsoid(glm::Vec3),
    Box(HitboxMesh),
}

/// contains all of the hitbox vertex data
pub(crate) struct HitboxMesh {
    vertices: Vec<glm::Vec3>,
    faces: Vec<[usize; 3]>,
}

impl HitboxMesh {
    /// creates a box mesh from reach dimensions
    fn box_from_dimensions(dim: &glm::Vec3) -> Self {
        Self {
            vertices: vec![
                glm::vec3(-dim.x, -dim.y, dim.z),
                glm::vec3(-dim.x, dim.y, dim.z),
                glm::vec3(-dim.x, -dim.y, -dim.z),
                glm::vec3(-dim.x, dim.y, -dim.z),
                glm::vec3(dim.x, -dim.y, dim.z),
                glm::vec3(dim.x, dim.y, dim.z),
                glm::vec3(dim.x, -dim.y, -dim.z),
                glm::vec3(dim.x, dim.y, -dim.z),
            ],
            faces: vec![
                [1, 2, 0],
                [3, 6, 2],
                [7, 4, 6],
                [5, 0, 4],
                [6, 0, 2],
                [3, 5, 7],
                [1, 3, 2],
                [3, 7, 6],
                [7, 5, 4],
                [5, 1, 0],
                [6, 4, 0],
                [3, 1, 5],
            ],
        }
    }

    /// creates a hitbox in the form of a convex hull of the mesh
    fn convex_hull(mut self) -> Self {
        assert!(
            self.vertices.len() >= 4,
            "mesh must be at least as complex as a tetrahedron for it to have a convex hull hitbox"
        );
        // TODO
        // find initial tetrahedron
        // for each face find the furthest point away from it
        // do this as long as there are points outside of the faces
        self
    }

    /// removes vertices that are not used by a face and corrects the face indices
    fn remove_unused_vertices(&mut self) {
        let mut used_indices = self.faces.iter().flatten().copied().collect::<HashSet<_>>();
        for i in 0..self.vertices.len() {
            while !used_indices.contains(&i) {
                self.vertices.swap_remove(i);
                self.faces
                    .iter_mut()
                    .flatten()
                    .filter(|index| **index == self.vertices.len())
                    .for_each(|index| *index = i);
                if used_indices.remove(&self.vertices.len()) {
                    used_indices.insert(i);
                }
            }
            if i == self.vertices.len() {
                break;
            }
        }
    }
}

/// represents one edge in a mesh with an error metric
#[derive(PartialEq)]
struct Edge {
    v1: usize,
    v2: usize,
    error: f32,
}

impl Eq for Edge {}

impl PartialOrd<Self> for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.error.partial_cmp(&other.error)
    }
}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Less)
    }
}
