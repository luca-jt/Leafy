use crate::glm;
use gl::types::*;
use obj::{load_obj, Obj, TexturedVertex};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// a mesh that can be rendered in gl
pub struct Mesh {
    pub(crate) positions: Vec<glm::Vec3>,
    pub(crate) texture_coords: Vec<glm::Vec2>,
    pub(crate) normals: Vec<glm::Vec3>,
    pub(crate) indeces: Vec<GLuint>,
}

impl Mesh {
    /// creates a new Mesh from an obj file
    pub(crate) fn new(file_path: impl AsRef<Path>) -> Self {
        // load scene data from file
        let data = BufReader::new(File::open(file_path).expect("file not found"));
        let model: Obj<TexturedVertex, GLuint> = load_obj(data).unwrap();

        // convert the data into the required format
        let positions: Vec<glm::Vec3> = model
            .vertices
            .iter()
            .map(|vertex| {
                glm::Vec3::new(vertex.position[0], vertex.position[1], vertex.position[2])
            })
            .collect();

        let texture_coords: Vec<glm::Vec2> = model
            .vertices
            .iter()
            .map(|vertex| glm::Vec2::new(vertex.texture[0], vertex.texture[1]))
            .collect();

        let normals: Vec<glm::Vec3> = model
            .vertices
            .iter()
            .map(|vertex| glm::Vec3::new(vertex.normal[0], vertex.normal[1], vertex.normal[2]))
            .collect();

        let indeces: Vec<GLuint> = model.indices;

        Self {
            positions,
            texture_coords,
            normals,
            indeces,
        }
    }

    /// yields the number of verteces per object
    pub fn num_verteces(&self) -> usize {
        self.positions.len()
    }

    /// yields the number of indeces per object
    pub fn num_indeces(&self) -> usize {
        self.indeces.len()
    }

    /// generates the inertia matrix for the mesh
    pub(crate) fn generate_interita_matrix(&self) -> glm::Mat3 {
        // TODO
        glm::Mat3::identity()
    }
}
