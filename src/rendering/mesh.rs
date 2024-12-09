use crate::ecs::component::Scale;
use crate::glm;
use crate::utils::constants::ORIGIN;
use crate::utils::tools::to_vec4;
use gl::types::*;
use itertools::Itertools;
use obj::{load_obj, Obj, TexturedVertex};
use petgraph::stable_graph::{NodeIndex, StableUnGraph};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::ops::{Index, IndexMut};
use std::path::Path;

/// vertex used in the AOS meshes
#[derive(Debug, Copy, Clone, Default)]
struct MeshVertex {
    position: glm::Vec3,
    uv: glm::Vec2,
    normal: glm::Vec3,
}

/// mesh containing vertex structs as opposed to the regular soa mesh which allows for easier processing
#[derive(Clone)]
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

        // SELECTING VALID PAIRS FOR THE CONSTRACTIONS
        // -> one of two cases: v1->v2 is edge or distance(v1, v2) < t with t being a threshold parameter
        // -> t = 0 would be equivalent to a regular edge contraction algo

        // CALCULATING THE ERRORS
        // the error for each vertex v = (x, y, z, 1) is the quadric form delta(v) = v^T*Q*v
        // initial matrices are constructed like this:
        //
        // for each vertex find all the triangles that meet at that vertex
        // for each triangle plane calculate p = [a, b, c, d]^T where the plane is defined by the equation ax + by + cz + d = 0 where a^2 + b^2 + c^2 = 1
        // that can be done using the plane normal vector n = (a, b, c) as the plane equation then is <(x, y, z), n> + d = 0
        // that means that d is the distance from the origin
        // the error quadric then becomes delta(v) = v^T * sum(K_p for p in planes) * v where K_p = p*p^T and sum(K_p for p in planes) = Q
        //
        // for each contraction we have to approximate the error at the new location of the merged vertices with Q_1 + Q_2 = Q_new
        // to find the new location of the produced vertex we find the minimum of the error function which is a linear problem:
        // sum of partial derivatives of the delta function for x, y, z shall be = 0
        //
        // that is equivalent to solving    |q11 q12 q13 q14 |
        //                                  |q12 q22 q23 q24 |
        //                                  |q13 q23 q33 q34 | * v_new = (0, 0, 0, 1)
        //                                  | 0   0   0   1  |
        //
        // which is the same as doing         |q11 q12 q13 q14 |^-1
        //                                    |q12 q22 q23 q24 |
        //                            v_new = |q13 q23 q33 q34 |   * (0, 0, 0, 1)
        //                                    | 0   0   0   1  |
        // if the matrix is invertible
        // if that is not the case we fall back to trying to find the optimal point along the segment v1 v2
        // if this also fails we fall back on choosing v_new from amongst the endpoints and the midpoint

        // SUMMARY:
        // 1. find all the valid vertex pairs
        // 2. compute the initial matrices Q
        // 3. compute the contraction target for each pair with contraction cost v_new^T * (Q_1 + Q_2) * v_new
        // 4. put all the in a heap keyed on cost with the minimum cost pair at the top
        // 5. iteratively remove the pair v1 v2 of least cost from the heap, contract this pair, and update the costs of all valid pairs involving v1

        let mut mesh_graph = MeshErrorGraph::from_edges(
            self.faces
                .iter()
                .map(|face| [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])])
                .flatten(),
        );

        // calculate the inital error matrix Q for a all vertices
        for i in 0..self.vertices.len() {
            let index = NodeIndex::new(i);
            let error_matrix = calculate_error_matrix(&mesh_graph, index);
            let vertex_data = mesh_graph.index_mut(index);
            *vertex_data = ErrorVertexData {
                mesh_vertex: self.vertices[i],
                error_matrix,
            };
        }

        let error_threshold = 0.2;
        let mut valid_pairs = find_all_valid_pairs(error_threshold, &mesh_graph);

        while self.faces.len() > target_triangle_count && !valid_pairs.is_empty() {
            let pair_to_contract = valid_pairs.pop().unwrap();
            contract_pair(pair_to_contract, &mut mesh_graph, &mut valid_pairs);
        }

        // reconstruct the mesh from the final graph data
        self.faces = convert_graph_edges_to_triangles(&mesh_graph);
        self.remove_unused_vertices();
        self
    }

    /// removes vertices that are not used by a face and corrects the face indices
    fn remove_unused_vertices(&mut self) {
        let mut used_indices = self.faces.iter().flatten().copied().collect::<HashSet<_>>();
        for i in 0..self.vertices.len() {
            while !used_indices.contains(&i) && i < self.vertices.len() {
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
            if i >= self.vertices.len() - 1 {
                break;
            }
        }
    }
}

/// converts the graph representation of the mesh into the triangle face representation
fn convert_graph_edges_to_triangles(mesh_graph: &MeshErrorGraph) -> Vec<[usize; 3]> {
    let triangles = HashSet::new();
    // TODO
    triangles.into_iter().collect_vec()
}

/// calculates the initital error matrix for a vertex at index
fn calculate_error_matrix(mesh_graph: &MeshErrorGraph, index: NodeIndex<usize>) -> glm::Mat4 {
    let mut accumulator = glm::Mat4::zeros();

    let incident_triangles = mesh_graph
        .neighbors(index)
        .tuple_combinations()
        .filter(|&(neighbor1, neighbor2)| mesh_graph.contains_edge(neighbor1, neighbor2))
        .map(|(neighbor1, neighbor2)| [index, neighbor1, neighbor2]);

    for triangle in incident_triangles {
        let vertex1 = mesh_graph.index(triangle[0]).mesh_vertex.position;
        let vertex2 = mesh_graph.index(triangle[1]).mesh_vertex.position;
        let vertex3 = mesh_graph.index(triangle[2]).mesh_vertex.position;
        let plane_normal = (vertex2 - vertex1).cross(&(vertex3 - vertex1)).normalize();
        let distance_from_origin = -plane_normal.dot(&vertex1);
        let mut p = to_vec4(&plane_normal);
        p.w = distance_from_origin;
        accumulator += p * p.transpose();
    }
    accumulator
}

/// find all vaild vertex pairs for contraction that are either edges or have a distance < error_threshold
fn find_all_valid_pairs(
    error_threshold: f32,
    mesh_graph: &MeshErrorGraph,
) -> BinaryHeap<ErrorVertexPair> {
    let mut valid_pairs = BinaryHeap::new();
    // add all edges as valid pairs
    for edge_idx in mesh_graph.edge_indices() {
        let (v1, v2) = mesh_graph.edge_endpoints(edge_idx).unwrap();
        add_valid_vertex_pair(v1, v2, mesh_graph, &mut valid_pairs);
    }
    // add all pairs that have a distance < error_threshold
    for v1 in mesh_graph.node_indices() {
        for v2 in mesh_graph.node_indices().skip(v1.index()) {
            if mesh_graph.contains_edge(v1, v2) {
                continue;
            }
            let pos1 = mesh_graph.index(v1).mesh_vertex.position;
            let pos2 = mesh_graph.index(v2).mesh_vertex.position;
            if glm::distance(&pos1, &pos2) <= error_threshold {
                add_valid_vertex_pair(v1, v2, mesh_graph, &mut valid_pairs);
            }
        }
    }
    valid_pairs
}

/// adds a new error vertex pair to the heap of valid pairs
fn add_valid_vertex_pair(
    v1: NodeIndex<usize>,
    v2: NodeIndex<usize>,
    mesh_graph: &MeshErrorGraph,
    valid_pairs: &mut BinaryHeap<ErrorVertexPair>,
) {
    let vertex1 = mesh_graph.index(v1);
    let vertex2 = mesh_graph.index(v2);

    let q_new = vertex1.error_matrix + vertex2.error_matrix; // new error matrix
    let partial_derivative_mat = glm::mat4(
        q_new.m11, q_new.m12, q_new.m13, q_new.m14, q_new.m12, q_new.m22, q_new.m23, q_new.m24,
        q_new.m13, q_new.m23, q_new.m33, q_new.m34, 0.0, 0.0, 0.0, 1.0,
    );
    let v_new = if let Some(inv_deriv_mat) = partial_derivative_mat.try_inverse() {
        inv_deriv_mat * glm::vec4(0.0, 0.0, 0.0, 1.0)
    } else {
        // fall back on choosing v_new as the midpoint
        to_vec4(&((vertex1.mesh_vertex.position + vertex2.mesh_vertex.position) / 2.0))
        // NOTE: the fallback to finding the optimal point along the segment v1 v2 is not used for perfomance
        // maybe this will be added in the future
    };

    let new_pair = ErrorVertexPair {
        v1,
        v2,
        error: (v_new.transpose() * q_new * v_new).x,
        v_new: v_new.xyz(),
    };
    valid_pairs.push(new_pair);
}

/// contracts a pair in the mesh simplification algorithm and modifies the relevant pairs and error data
fn contract_pair(
    pair: ErrorVertexPair,
    mesh_graph: &mut MeshErrorGraph,
    valid_pairs: &mut BinaryHeap<ErrorVertexPair>,
) {
    // move v1 to new position stored in the pair
    let vertex1 = *mesh_graph.index(pair.v1);
    let vertex2 = *mesh_graph.index(pair.v2);
    let v1_ref = mesh_graph.index_mut(pair.v1);

    v1_ref.mesh_vertex.position = pair.v_new;
    let shift_transform = glm::Mat3::from_diagonal(&glm::vec3(
        vertex1.mesh_vertex.position.x / vertex2.mesh_vertex.position.x,
        vertex1.mesh_vertex.position.y / vertex2.mesh_vertex.position.y,
        vertex1.mesh_vertex.position.z / vertex2.mesh_vertex.position.z,
    )); // this is done in case a point other than the mid point is chosen
    v1_ref.mesh_vertex.normal = shift_transform * vertex1.mesh_vertex.normal;
    v1_ref.mesh_vertex.uv = (vertex1.mesh_vertex.uv + vertex2.mesh_vertex.uv) / 2.0;
    v1_ref.error_matrix = vertex1.error_matrix + vertex2.error_matrix;

    // connect all of v2's edges to v1
    let connected_to_v2 = mesh_graph
        .neighbors(pair.v2)
        .filter(|neighbor| *neighbor != pair.v1)
        .collect_vec();

    for node in connected_to_v2 {
        mesh_graph.add_edge(pair.v1, node, ());
    }
    // delete v2
    mesh_graph.remove_node(pair.v2);

    // -> when contracting a set of vertices, also the valid pair partners are combined
    // -> recompute some pairs as the merged location might not be the same as the old location of v1

    // remove all pairs containing v1 or v2
    valid_pairs
        .retain(|p| p.v1 != pair.v1 && p.v1 != pair.v2 && p.v2 != pair.v1 && p.v2 != pair.v2);
    // add new pairs for v1
    for neighbor in mesh_graph.neighbors(pair.v1) {
        add_valid_vertex_pair(pair.v1, neighbor, &mesh_graph, valid_pairs);
        // NOTE: at this point we dont add the ones with the error threshold for performance reasons
    }
}

/// used for the graph representation of a mesh that is required for some algorithmic things
type MeshErrorGraph = StableUnGraph<ErrorVertexData, (), usize>;

#[derive(Debug, Default, Copy, Clone)]
struct ErrorVertexData {
    mesh_vertex: MeshVertex,
    error_matrix: glm::Mat4,
}

/// stores one vertex pair with error data that is used in the mesh simplification algorithm
#[derive(Debug, Copy, Clone)]
struct ErrorVertexPair {
    v1: NodeIndex<usize>,
    v2: NodeIndex<usize>,
    error: f32,
    v_new: glm::Vec3,
}

impl PartialEq<Self> for ErrorVertexPair {
    fn eq(&self, other: &Self) -> bool {
        // we want the error pairs to be unordered, so this accounts for that
        self.v1 == other.v1 && self.v2 == other.v2 || self.v1 == other.v2 && self.v2 == other.v1
    }
}

impl Eq for ErrorVertexPair {}

impl PartialOrd<Self> for ErrorVertexPair {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.error.partial_cmp(&self.error)
    }
}

impl Ord for ErrorVertexPair {
    fn cmp(&self, other: &Self) -> Ordering {
        // this is for a max heap so the implementation has to account for that if we want to have a min heap for errors
        other.error.partial_cmp(&self.error).unwrap()
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
    pub(crate) fn generate_lods(&self) -> [Mesh; 4] {
        // TODO: do all of the lods in one iteration of the algorithm -> more performant
        let lod1 = self.aos_mesh().simplified();
        let lod2 = lod1.clone().simplified();
        let lod3 = lod2.clone().simplified();
        let lod4 = lod3.clone().simplified();
        [
            lod1.to_mesh(),
            lod2.to_mesh(),
            lod3.to_mesh(),
            lod4.to_mesh(),
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
        let (a, b, c, d) = self.find_initial_tetrahedron();
        self.faces = vec![[a, b, c], [b, a, d], [d, a, c], [b, d, c]];

        // find the points outside the initial tetrahedron
        let mut outside_points =
            self.find_outside_points(self.faces.iter(), (0..self.vertices.len()).collect());

        while outside_points.iter().any(|p| !p.is_empty()) {
            let mut new_faces = vec![None; outside_points.len()]; // indices are face indices to delete after every iteration
            for (i, points) in outside_points.iter().enumerate() {
                if !points.is_empty() {
                    let [a, b, c] = self.faces[i];
                    let (v1, v2, v3) = (self.vertices[a], self.vertices[b], self.vertices[c]);
                    let face_normal = (v2 - v1).cross(&(v3 - v1));
                    // find furthest point
                    let point = points
                        .iter()
                        .map(|j| (j, self.vertices[*j] - v1))
                        .map(|(j, v)| (j, face_normal.dot(&v)))
                        .filter(|(_, dotp)| *dotp > 0.0)
                        .max_by(|(_, dp1), (_, dp2)| dp1.partial_cmp(dp2).unwrap())
                        .map(|(j, _)| *j)
                        .unwrap();
                    // create new faces with the point
                    new_faces[i] = Some([[c, point, a], [b, point, c], [a, point, b]]);
                }
            }
            // the order of the outside sets and the faces is preserved as they are both mutated the same way
            for idx in (0..new_faces.len()).rev() {
                if let Some(faces) = new_faces[idx] {
                    // delete old faces
                    self.faces.swap_remove(idx);
                    let mut outside_set = outside_points.swap_remove(idx);
                    outside_set.retain(|p| faces.iter().flatten().all(|fp| fp != p));
                    // add new faces
                    self.faces.push(faces[0]);
                    self.faces.push(faces[1]);
                    self.faces.push(faces[2]);
                    // construct 3 outside sets from a given point cloud the same way it was done in the beginning
                    let new_outside_points = self.find_outside_points(faces.iter(), outside_set);
                    debug_assert_eq!(new_outside_points.len(), 3);
                    outside_points.extend(new_outside_points);
                }
            }
        }
        self.remove_unused_vertices();
        self
    }

    /// find the points outside of each face
    fn find_outside_points<'a>(
        &self,
        faces: impl Iterator<Item = &'a [usize; 3]>,
        point_cloud: Vec<usize>,
    ) -> Vec<Vec<usize>> {
        let faces = faces.collect_vec();
        let mut point_sets = Vec::with_capacity(faces.len());
        let mut points_used = HashSet::new();

        for face in faces {
            let (v1, v2, v3) = (
                self.vertices[face[0]],
                self.vertices[face[1]],
                self.vertices[face[2]],
            );
            let face_normal = (v2 - v1).cross(&(v3 - v1));

            let fitting_points = point_cloud
                .iter()
                .filter(|i| !points_used.contains(*i))
                .map(|i| (i, self.vertices[*i] - v1))
                .map(|(i, v)| (i, face_normal.dot(&v)))
                .filter(|(_, dotp)| *dotp > 0.0)
                .map(|(i, _)| *i)
                .collect_vec();

            for fitting in fitting_points.iter() {
                points_used.insert(*fitting);
            }
            point_sets.push(fitting_points);
        }
        point_sets
    }

    /// find the base tetrahedron for the convex hull algorithm
    fn find_initial_tetrahedron(&self) -> (usize, usize, usize, usize) {
        // find two extreme points along x axis
        let (min_x_idx, max_x_idx) =
            self.vertices
                .iter()
                .enumerate()
                .fold((0, 0), |(min_idx, max_idx), (i, v)| {
                    let mut updated = (min_idx, max_idx);
                    if v.x < self.vertices[min_idx].x {
                        updated.0 = i;
                    }
                    if v.x > self.vertices[max_idx].x {
                        updated.1 = i;
                    }
                    updated
                });
        // find the point farthest from the line formed by the two points
        let line_vec = self.vertices[max_x_idx] - self.vertices[min_x_idx];
        let third_idx = self
            .vertices
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != min_x_idx && i != max_x_idx)
            .map(|(i, v)| {
                (
                    i,
                    line_vec
                        .cross(&(v - self.vertices[min_x_idx]))
                        .norm_squared(),
                )
            })
            .max_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
            .unwrap()
            .0;
        // find the fourth point that forms a valid tetrahedron
        // we have to check for the max to be greater than 0 because the we want to make shure the triangles are in CCW order
        // taking the min value in in the absence of a positive value means the indeces sould be ordered differently
        let normal = (self.vertices[third_idx] - self.vertices[min_x_idx]).cross(&line_vec);
        let fourth_options = self
            .vertices
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != min_x_idx && i != max_x_idx && i != third_idx)
            .fold(
                (None, ORIGIN, None, ORIGIN),
                |(mut min_idx, mut min_vec, mut max_idx, mut max_vec), (i, v)| {
                    let dotp_to_compare = v.dot(&normal);
                    if dotp_to_compare > max_vec.dot(&normal) {
                        max_idx = Some(i);
                        max_vec = *v;
                    }
                    if dotp_to_compare < min_vec.dot(&normal) {
                        min_idx = Some(i);
                        min_vec = *v;
                    }
                    (min_idx, min_vec, max_idx, max_vec)
                },
            );
        if let Some(fourth_idx) = fourth_options.2 {
            (min_x_idx, max_x_idx, third_idx, fourth_idx)
        } else if let Some(fourth_idx) = fourth_options.0 {
            (max_x_idx, min_x_idx, third_idx, fourth_idx)
        } else {
            // in this case other points are only on the same plane -> just return the first one that exists
            let fourth_idx = (0..self.vertices.len())
                .filter(|&i| i != min_x_idx && i != max_x_idx && i != third_idx)
                .next()
                .unwrap();
            (min_x_idx, max_x_idx, third_idx, fourth_idx)
        }
    }

    /// removes vertices that are not used by a face and corrects the face indices
    fn remove_unused_vertices(&mut self) {
        let mut used_indices = self.faces.iter().flatten().copied().collect::<HashSet<_>>();
        for i in 0..self.vertices.len() {
            while !used_indices.contains(&i) && i < self.vertices.len() {
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
            if i >= self.vertices.len() - 1 {
                break;
            }
        }
    }
}
