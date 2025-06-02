use crate::internal_prelude::*;
use petgraph::stable_graph::{NodeIndex, StableUnGraph};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::io::BufReader;
use std::ops::{Index, IndexMut};
use tobj::{load_obj_buf, GPU_LOAD_OPTIONS};

/// identifier for a triangle in the AOS mesh, maps to triangles that only use the unique vertices
type TriangleID = u64;

/// mesh containing vertex structs as opposed to the regular SOA mesh which allows for easier processing
#[derive(Debug, Clone)]
struct AlgorithmMesh {
    name: String,
    source_file: Rc<Path>,
    material_name: Option<String>,
    vertices: Vec<Vec3>,
    faces: Vec<[usize; 3]>,
    triangle_map: AHashMap<usize, AHashSet<TriangleID>>,
    windings: AHashMap<TriangleID, [usize; 3]>,
}

impl AlgorithmMesh {
    /// converts back to a regular mesh
    fn into_mesh(self) -> Mesh {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();

        for face in self.faces.iter() {
            let position1 = self.vertices[face[0]];
            let position2 = self.vertices[face[1]];
            let position3 = self.vertices[face[2]];

            positions.extend_from_slice(&[position1, position2, position3]);

            let normal = (position2 - position1)
                .cross(&(position3 - position1))
                .normalize();

            normals.extend_from_slice(&[normal, normal, normal]);

            indices.extend_from_slice(&[
                (positions.len() - 3) as GLuint,
                (positions.len() - 2) as GLuint,
                (positions.len() - 1) as GLuint,
            ]);
        }
        let max_reach = self
            .vertices
            .iter()
            .map(|v| v.abs())
            .fold(ORIGIN, |mut current, p| {
                current.x = current.x.max(p.x);
                current.y = current.y.max(p.y);
                current.z = current.z.max(p.z);
                current
            });

        // TODO: we might want to keep track of these vertex attributes in the future in mesh manipulation
        let texture_coords = vec![vec2(0.0, 0.0); positions.len()];
        let colors = vec![vec4(1.0, 1.0, 1.0, 1.0); positions.len()];
        let tangents = vec![vec3(0.0, 0.0, 0.0); positions.len()];

        Mesh {
            name: self.name,
            source_file: self.source_file,
            positions,
            colors,
            normals,
            texture_coords,
            indices,
            tangents,
            max_reach,
            material_name: self.material_name,
        }
    }

    /// creates a hitbox in the form of a unaltered version of the mesh
    fn hitbox_mesh(self) -> HitboxMesh {
        HitboxMesh {
            vertices: self.vertices,
            faces: self.faces,
        }
    }

    /// creates a simplified version of the mesh that is used for LOD and hitboxes
    fn simplified(mut self) -> Self {
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
        // 4. put all the pairs in a heap keyed on cost with the minimum cost pair at the top
        // 5. iteratively remove the pair v1 v2 of least cost from the heap, contract this pair, and update the costs of all valid pairs involving v1

        let target_node_count = (self.vertices.len() / 2).max(4);

        let edges = self
            .faces
            .iter()
            .flat_map(|&face| [[face[0], face[1]], [face[1], face[2]], [face[2], face[0]]])
            .map(|mut edge| {
                edge.sort_unstable();
                edge
            })
            .unique();

        let mut mesh_graph = MeshErrorGraph::default();

        // add vertices with positions
        for vertex in self.vertices.iter() {
            mesh_graph.add_node(ErrorVertex {
                position: *vertex,
                error_matrix: Mat4::default(),
            });
        }

        // add unique edges
        for (node1, node2) in edges
            .into_iter()
            .map(|[n1, n2]| (NodeIndex::new(n1), NodeIndex::new(n2)))
        {
            mesh_graph.add_edge(node1, node2, ());
        }

        // calculate the inital error matrix Q for a all vertices
        for i in 0..self.vertices.len() {
            let index = NodeIndex::new(i);
            let error_matrix = calculate_error_matrix(&mesh_graph, index);
            let vertex_data = mesh_graph.index_mut(index);
            vertex_data.error_matrix = error_matrix;
        }

        let error_threshold = 0.02;
        let mut valid_pairs = find_all_valid_pairs(error_threshold, &mesh_graph);

        while mesh_graph.node_count() > target_node_count && !valid_pairs.is_empty() {
            let pair_to_contract = valid_pairs.pop().unwrap();
            self.contract_pair(pair_to_contract, &mut mesh_graph, &mut valid_pairs);
        }

        // reconstruct the mesh from the final graph data
        self.convert_graph_edges_to_triangles(&mesh_graph);
        self.convert_graph_nodes_to_vertices(&mesh_graph);
        self.remove_unused_vertices();
        self
    }

    /// removes vertices that are not used by a face and corrects the face indices
    fn remove_unused_vertices(&mut self) {
        let mut used_indices = self
            .faces
            .iter()
            .flatten()
            .copied()
            .collect::<AHashSet<_>>();
        for i in 0..self.vertices.len() {
            while !used_indices.contains(&i) && i < self.vertices.len() - 1 {
                self.vertices.swap_remove(i);
                self.faces
                    .iter_mut()
                    .flatten()
                    .filter(|index| **index == self.vertices.len())
                    .for_each(|index| *index = i);

                if used_indices.remove(&self.vertices.len()) {
                    used_indices.insert(i);
                    let ids = self.triangle_map.remove(&self.vertices.len()).unwrap();
                    for id in ids.iter() {
                        if let Some(winding) = self.windings.remove(id) {
                            self.windings.insert(*id, winding);
                        }
                    }
                    self.triangle_map.insert(i, ids);
                }
            }
            debug_assert!(!self.vertices.is_empty());
        }
    }

    /// converts the nodes currently stored in the graph to the mesh vertex positions
    fn convert_graph_nodes_to_vertices(&mut self, mesh_graph: &MeshErrorGraph) {
        for node_idx in mesh_graph.node_indices() {
            let vertex = mesh_graph.index(node_idx).position;
            self.vertices[node_idx.index()] = vertex;
        }
    }

    /// yields the triangle id for a given set of indices
    fn triangle_id_from_indices(&self, face: [usize; 3]) -> Option<TriangleID> {
        // @Cleanup: there seem to be some cases where there are triangles created that should not exist
        // to fix this temporarily we just skip them and return None
        // this is kind of dirty and should be cleaned up later on
        // there might be a bigger underlying issue associated with that

        let triangles1 = self.triangle_map.get(&face[0]).unwrap();
        let triangles2 = self.triangle_map.get(&face[1]).unwrap();
        let triangles3 = self.triangle_map.get(&face[2]).unwrap();

        let mut common_id_iter = triangles1
            .iter()
            .filter(|&id| triangles2.contains(id) && triangles3.contains(id));

        let triangle_id = *common_id_iter.next()?;
        assert!(
            common_id_iter.next().is_none(),
            "more than one common triangle id"
        );
        Some(triangle_id)
    }

    /// converts the graph representation of the mesh into the triangle face representation
    fn convert_graph_edges_to_triangles(&mut self, mesh_graph: &MeshErrorGraph) {
        self.faces = mesh_graph
            .edge_indices()
            .map(|idx| mesh_graph.edge_endpoints(idx).unwrap())
            .flat_map(|(node1, node2)| {
                mesh_graph
                    .neighbors(node1)
                    .filter(|nb| mesh_graph.contains_edge(node2, *nb))
                    .filter_map(|nb| {
                        let triangle = [node1.index(), node2.index(), nb.index()];
                        let triangle_id = self.triangle_id_from_indices(triangle)?;
                        let triangle_correct_winding = *self.windings.get(&triangle_id).unwrap();
                        Some(triangle_correct_winding)
                    })
                    .collect_vec()
            })
            .unique()
            .collect_vec();
    }

    /// contracts a pair in the mesh simplification algorithm and modifies the relevant pairs and error data
    fn contract_pair(
        &mut self,
        pair: ErrorVertexPair,
        mesh_graph: &mut MeshErrorGraph,
        valid_pairs: &mut BinaryHeap<ErrorVertexPair>,
    ) {
        // prevent two faces folding in on each other
        if mesh_graph
            .neighbors(pair.v1)
            .filter(|&nb| nb != pair.v2)
            .tuple_combinations()
            .filter(|&(nb1, nb2)| mesh_graph.contains_edge(nb1, nb2))
            .any(|(nb1, nb2)| {
                mesh_graph.contains_edge(nb1, pair.v2) && mesh_graph.contains_edge(nb2, pair.v2)
            })
        {
            return;
        }

        // for all triangles that are effectively deleted remove entries from triangle map and manifestations (only relevant for pairs that are edges)
        if mesh_graph.contains_edge(pair.v1, pair.v2) {
            for neighbor in mesh_graph
                .neighbors(pair.v1)
                .filter(|&nb| mesh_graph.contains_edge(nb, pair.v2))
            {
                if let Some(triangle_id) = self.triangle_id_from_indices([
                    pair.v1.index(),
                    pair.v2.index(),
                    neighbor.index(),
                ]) {
                    let triangles_v1 = self.triangle_map.get_mut(&pair.v1.index()).unwrap();
                    assert!(triangles_v1.remove(&triangle_id));
                    let triangles_v2 = self.triangle_map.get_mut(&pair.v2.index()).unwrap();
                    assert!(triangles_v2.remove(&triangle_id));
                    let triangles_neighbor = self.triangle_map.get_mut(&neighbor.index()).unwrap();
                    assert!(triangles_neighbor.remove(&triangle_id));

                    self.windings.remove(&triangle_id).unwrap();
                }
            }
        }

        // move v1 to new position stored in the pair
        let vertex1 = *mesh_graph.index(pair.v1);
        let vertex2 = *mesh_graph.index(pair.v2);
        let v1_ref = mesh_graph.index_mut(pair.v1);

        v1_ref.position = pair.v_new;
        v1_ref.error_matrix = vertex1.error_matrix + vertex2.error_matrix;

        // exchange v2 for v1 in the windings of triangles that it was a part of
        for id in self.triangle_map.get(&pair.v2.index()).unwrap().iter() {
            let triangle = self.windings.get_mut(id).unwrap();
            debug_assert!(!triangle.contains(&pair.v1.index()));
            let index_to_switch = triangle
                .iter_mut()
                .find(|i| **i == pair.v2.index())
                .unwrap();
            *index_to_switch = pair.v1.index();
        }

        // delete v2's associated triangles and insert them into the ones for v1
        let triangles_of_v2 = self.triangle_map.remove(&pair.v2.index()).unwrap();
        self.triangle_map
            .get_mut(&pair.v1.index())
            .unwrap()
            .extend(triangles_of_v2);

        // remove all pairs containing v1 or v2
        valid_pairs
            .retain(|p| p.v1 != pair.v1 && p.v2 != pair.v1 && p.v1 != pair.v2 && p.v2 != pair.v2);

        // connect all of v2's triangle edges to v1
        let connected_to_v2 = mesh_graph
            .neighbors(pair.v2)
            .filter(|&neighbor| neighbor != pair.v1)
            .filter(|&neighbor| !mesh_graph.contains_edge(neighbor, pair.v1))
            .collect_vec();

        for node in connected_to_v2 {
            mesh_graph.add_edge(pair.v1, node, ());
        }

        // delete v2
        mesh_graph.remove_node(pair.v2).unwrap();

        // compute new valid pairs
        for neighbor in mesh_graph.neighbors(pair.v1) {
            add_valid_vertex_pair(pair.v1, neighbor, mesh_graph, valid_pairs);
            // NOTE: at this point we dont add the ones with the error threshold for performance reasons -> TODO?
        }
    }
}

/// calculates the initital error matrix for a vertex at index
fn calculate_error_matrix(mesh_graph: &MeshErrorGraph, index: NodeIndex<usize>) -> Mat4 {
    let mut accumulator = Mat4::zeros();

    let incident_triangles = mesh_graph
        .neighbors(index)
        .tuple_combinations()
        .filter(|&(neighbor1, neighbor2)| mesh_graph.contains_edge(neighbor1, neighbor2))
        .map(|(neighbor1, neighbor2)| [index, neighbor1, neighbor2]);

    for triangle in incident_triangles {
        let vertex1 = mesh_graph.index(triangle[0]).position;
        let vertex2 = mesh_graph.index(triangle[1]).position;
        let vertex3 = mesh_graph.index(triangle[2]).position;
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
    for (v1, v2) in mesh_graph.node_indices().tuple_combinations() {
        if mesh_graph.contains_edge(v1, v2) {
            continue;
        }
        let pos1 = mesh_graph.index(v1).position;
        let pos2 = mesh_graph.index(v2).position;
        if glm::distance(&pos1, &pos2) <= error_threshold {
            add_valid_vertex_pair(v1, v2, mesh_graph, &mut valid_pairs);
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
    let partial_derivative_mat = Mat4::from_columns(&[
        vec4(q_new.m11, q_new.m12, q_new.m13, 0.0),
        vec4(q_new.m12, q_new.m22, q_new.m23, 0.0),
        vec4(q_new.m13, q_new.m23, q_new.m33, 0.0),
        vec4(q_new.m14, q_new.m24, q_new.m34, 1.0),
    ]);

    let v_new = if let Some(inv_deriv_mat) = partial_derivative_mat.try_inverse() {
        inv_deriv_mat * vec4(0.0, 0.0, 0.0, 1.0)
    } else {
        // fall back on choosing v_new as the midpoint
        to_vec4(&((vertex1.position + vertex2.position) / 2.0))
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

/// used for the graph representation of a mesh that is required for some algorithmic things
type MeshErrorGraph = StableUnGraph<ErrorVertex, (), usize>;

/// vertex data for one vertex in the mesh graph that is used for simplifying meshes
#[derive(Debug, Default, Copy, Clone)]
struct ErrorVertex {
    position: Vec3,
    error_matrix: Mat4,
}

/// stores one vertex pair with error data that is used in the mesh simplification algorithm
#[derive(Debug, Copy, Clone, PartialEq)]
struct ErrorVertexPair {
    v1: NodeIndex<usize>,
    v2: NodeIndex<usize>,
    error: f32,
    v_new: Vec3,
}

impl Eq for ErrorVertexPair {}

#[allow(clippy::non_canonical_partial_ord_impl)]
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
#[derive(Clone)]
pub(crate) struct Mesh {
    pub(crate) name: String,
    pub(crate) source_file: Rc<Path>,
    pub(crate) positions: Vec<Vec3>,
    pub(crate) colors: Vec<Vec4>,
    pub(crate) normals: Vec<Vec3>,
    pub(crate) texture_coords: Vec<Vec2>,
    pub(crate) indices: Vec<GLuint>,
    pub(crate) tangents: Vec<Vec3>,
    pub(crate) max_reach: Vec3,
    pub(crate) material_name: Option<String>, // the presence of this means the material source can be inherited
}

impl Mesh {
    /// creates a new Mesh from a byte array
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        let mut data = BufReader::new(bytes);
        let (models, _) = load_obj_buf(&mut data, &GPU_LOAD_OPTIONS, |_| unreachable!()).unwrap();
        Self::from_obj_data(&models[0], Path::new("internal").into(), None)
    }

    /// loads a mesh from loaded object file data
    #[rustfmt::skip]
    pub(crate) fn from_obj_data(model: &tobj::Model, source_file: Rc<Path>, material_name: Option<String>) -> Self {
        let obj = &model.mesh;

        let positions = obj.positions.iter().copied().tuples().map(|(x, y, z)| vec3(x, y, z)).collect_vec();
        let indices = obj.indices.clone();

        let colors = if obj.vertex_color.is_empty() {
            vec![vec4(1.0, 1.0, 1.0, 1.0); positions.len()]
        } else {
            obj.vertex_color.iter().copied().tuples().map(|(r, g, b)| vec4(r, g, b, 1.0)).collect_vec()
        };

        let normals = if obj.normals.is_empty() {
            let mut computed = vec![ORIGIN; positions.len()];
            for (a, b, c) in indices.iter().copied().tuples() {
                let p1 = positions[a as usize];
                let p2 = positions[b as usize];
                let p3 = positions[c as usize];
                let normal = (p2 - p1).cross(&(p3 - p1)).normalize();
                computed[a as usize] = normal;
                computed[b as usize] = normal;
                computed[c as usize] = normal;
            }
            computed
        } else {
            obj.normals.iter().copied().tuples().map(|(x, y, z)| vec3(x, y, z)).collect_vec()
        };

        let texture_coords = if obj.texcoords.is_empty() {
            vec![vec2(0.0, 0.0); positions.len()]
        } else {
            obj.texcoords.iter().copied().tuples().map(|(u, v)| vec2(u, v)).collect_vec()
        };

        let mut tangents = vec![vec3(0.0, 0.0, 0.0); positions.len()];
        for (a, b, c) in indices.iter().copied().tuples() {
            let p1 = positions[a as usize];
            let p2 = positions[b as usize];
            let p3 = positions[c as usize];
            let uv1 = texture_coords[a as usize];
            let uv2 = texture_coords[b as usize];
            let uv3 = texture_coords[c as usize];

            let edge1 = p2 - p1;
            let edge2 = p3 - p1;
            let delta_uv1 = uv2 - uv1;
            let delta_uv2 = uv3 - uv1;

            let f = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

            let mut tangent = vec3(0.0, 0.0, 0.0);
            tangent.x = f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x);
            tangent.y = f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y);
            tangent.z = f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z);

            tangents[a as usize] = tangent;
            tangents[b as usize] = tangent;
            tangents[c as usize] = tangent;
        }

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

        let name = model.name.clone();

        Self {
            name,
            source_file,
            positions,
            colors,
            normals,
            texture_coords,
            indices,
            tangents,
            max_reach,
            material_name
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

    /// generates the AOS based algorithm mesh for easier data parsing
    fn algorithm_mesh(&self) -> AlgorithmMesh {
        let mut original_mesh_faces = vec![[0, 0, 0]; self.indices.len() / 3];
        for (i, index) in self.indices.iter().enumerate() {
            original_mesh_faces[i / 3][i % 3] = *index as usize;
        }

        let mut vertices: Vec<Vec3> = Vec::new();
        let mut faces = Vec::with_capacity(original_mesh_faces.len());
        let mut triangle_map: AHashMap<usize, AHashSet<TriangleID>> = AHashMap::new();
        let mut windings: AHashMap<TriangleID, [usize; 3]> = AHashMap::new();

        for (i, face) in original_mesh_faces.into_iter().enumerate() {
            let mut aos_indices = [0_usize; 3];
            let triangle_id = i as TriangleID;

            for (i, index) in face.into_iter().enumerate() {
                let position = self.positions[index];

                let vertex_index = vertices
                    .iter()
                    .enumerate()
                    .find(|&(_, point)| *point == position)
                    .map(|(i, _)| i)
                    .unwrap_or_else(|| {
                        vertices.push(position);
                        vertices.len() - 1
                    });

                aos_indices[i] = vertex_index;

                triangle_map
                    .entry(vertex_index)
                    .or_default()
                    .insert(triangle_id);
            }
            assert!(windings.insert(triangle_id, aos_indices).is_none());
            faces.push(aos_indices);
        }

        AlgorithmMesh {
            name: self.name.clone(),
            source_file: self.source_file.clone(),
            material_name: self.material_name.clone(),
            vertices,
            faces,
            triangle_map,
            windings,
        }
    }

    /// generates the inverse inertia tensor matrix, center of mass and the mass
    pub(crate) fn intertia_data(&self, density: f32, scale: &Scale) -> (Mat3, Vec3, f32) {
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
            let scaled1 = mult_mat4_vec3(&scale_matrix, &pos1);
            let scaled2 = mult_mat4_vec3(&scale_matrix, &pos2);
            let scaled3 = mult_mat4_vec3(&scale_matrix, &pos3);
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
        iap = density * iap / 120.0 - mass * center_of_mass.y * center_of_mass.z;
        ibp = density * ibp / 120.0 - mass * center_of_mass.x * center_of_mass.y;
        icp = density * icp / 120.0 - mass * center_of_mass.x * center_of_mass.z;

        (
            Mat3::from_columns(&[
                vec3(ia, -ibp, -icp),
                vec3(-ibp, ib, -iap),
                vec3(-icp, -iap, ic),
            ])
            .try_inverse()
            .unwrap(),
            center_of_mass,
            mass,
        )
    }

    /// generates all the simpified meshes for the lod levels
    pub(crate) fn generate_lods(&self) -> [Mesh; 4] {
        let lod1 = self.algorithm_mesh().simplified().into_mesh();
        let lod2 = lod1.algorithm_mesh().simplified().into_mesh();
        let lod3 = lod2.algorithm_mesh().simplified().into_mesh();
        let lod4 = lod3.algorithm_mesh().simplified().into_mesh();
        [lod1, lod2, lod3, lod4]
    }

    /// generates the meshes' hitbox for the given hitbox type
    #[rustfmt::skip]
    pub(crate) fn generate_hitbox(&self, hitbox: &HitboxType) -> Hitbox {
        match hitbox {
            HitboxType::ConvexHull => Hitbox::ConvexMesh(self.algorithm_mesh().hitbox_mesh().convex_hull()),
            HitboxType::SimplifiedConvexHull => Hitbox::ConvexMesh(self.algorithm_mesh().simplified().hitbox_mesh().convex_hull()),
            HitboxType::Sphere => Hitbox::Sphere(self.max_reach.max()),
            HitboxType::Box => Hitbox::ConvexMesh(HitboxMesh::box_from_dims(&self.max_reach)),
        }
    }
}

/// computes the inertia moment for a given traingle and index
fn inertia_moment(triangle: &(Vec3, Vec3, Vec3), i: usize) -> f32 {
    triangle.0[i] * triangle.0[i]
        + triangle.1[i] * triangle.2[i]
        + triangle.1[i] * triangle.1[i]
        + triangle.0[i] * triangle.2[i]
        + triangle.2[i] * triangle.2[i]
        + triangle.0[i] * triangle.1[i]
}

/// computes the inertia product for a given traingle and indices
fn inertia_product(triangle: &(Vec3, Vec3, Vec3), i: usize, j: usize) -> f32 {
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

/// all possible versions of hitboxes
pub(crate) enum Hitbox {
    ConvexMesh(HitboxMesh),
    Sphere(f32),
}

impl Hitbox {
    /// creates a generic hitbox that is independant of mesh data
    pub(crate) fn from_generic_type(hitbox_type: HitboxType) -> Self {
        match hitbox_type {
            HitboxType::Sphere => Self::Sphere(1.0),
            _ => Self::ConvexMesh(HitboxMesh::box_from_dims(&Vec3::from_element(1.0))),
        }
    }
}

/// contains all of the hitbox vertex data
pub(crate) struct HitboxMesh {
    pub(crate) vertices: Vec<Vec3>,
    pub(crate) faces: Vec<[usize; 3]>,
}

impl HitboxMesh {
    /// creates a box mesh from reach dimensions
    fn box_from_dims(dim: &Vec3) -> Self {
        Self {
            vertices: vec![
                vec3(-dim.x, -dim.y, dim.z),
                vec3(-dim.x, dim.y, dim.z),
                vec3(-dim.x, -dim.y, -dim.z),
                vec3(-dim.x, dim.y, -dim.z),
                vec3(dim.x, -dim.y, dim.z),
                vec3(dim.x, dim.y, dim.z),
                vec3(dim.x, -dim.y, -dim.z),
                vec3(dim.x, dim.y, -dim.z),
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
        let mut points_used = AHashSet::new();

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
                .find(|&i| i != min_x_idx && i != max_x_idx && i != third_idx)
                .unwrap();
            (min_x_idx, max_x_idx, third_idx, fourth_idx)
        }
    }

    /// removes vertices that are not used by a face and corrects the face indices
    fn remove_unused_vertices(&mut self) {
        let mut used_indices = self
            .faces
            .iter()
            .flatten()
            .copied()
            .collect::<AHashSet<_>>();
        for i in 0..self.vertices.len() {
            while !used_indices.contains(&i) && i < self.vertices.len() - 1 {
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
            debug_assert!(!self.vertices.is_empty());
        }
    }
}
