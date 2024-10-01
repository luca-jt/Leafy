use crate::glm;
use crate::utils::file::get_model_path;
use gl::types::*;
use obj::{load_obj, Obj, TexturedVertex};
use std::fs::File;
use std::io::BufReader;

/// a mesh that can be rendered in gl
pub struct Mesh {
    pub(crate) positions: Vec<glm::Vec3>,
    pub(crate) texture_coords: Vec<glm::Vec2>,
    pub(crate) normals: Vec<glm::Vec3>,
    pub(crate) indeces: Vec<GLuint>,
}

impl Mesh {
    /// creates a new Mesh from an obj file
    pub(crate) fn new(file_name: &str) -> Self {
        // load scene data from file
        let data = BufReader::new(File::open(get_model_path(file_name)).expect("file not found"));
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

    /*pub fn generate_aos(&self, mesh_attribute: MeshAttribute) -> Vec<Vertex> {
        self.positions
            .iter()
            .zip(self.texture_coords.iter())
            .zip(self.normals.iter())
            .map(|data| Vertex {
                position: *data.0 .0,
                color: mesh_attribute
                    .color()
                    .map(|color32| color32.to_vec4())
                    .unwrap_or(Color32::WHITE.to_vec4()),
                uv_coords: *data.0 .1,
                normal: *data.1,
                tex_index: 0.0,
            })
            .collect()
    }*/
}
