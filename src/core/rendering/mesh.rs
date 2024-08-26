use gl::types::*;
use nalgebra_glm as glm;
use obj::{load_obj, Obj, TexturedVertex};
use std::cell::{Ref, RefCell};
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

use super::data::Vertex;
use crate::utils::file::get_model_path;

/// a mesh that can be rendered in gl
pub struct Mesh {
    pub(crate) positions: Vec<glm::Vec3>,
    pub(crate) texture_coords: Vec<glm::Vec2>,
    pub(crate) normals: Vec<glm::Vec3>,
    pub(crate) indeces: Vec<GLushort>,
}

impl Mesh {
    /// creates a new Mesh from an obj file
    pub(crate) fn new(file_name: &str) -> Self {
        // load scene data from file
        let data =
            BufReader::new(File::open(get_model_path(file_name)).expect("model file not found"));
        let model: Obj<TexturedVertex> = load_obj(data).unwrap();

        // convert the data into the required format
        let positions: Vec<glm::Vec3> = model
            .vertices
            .iter()
            .map(|vertex| {
                glm::Vec3::new(vertex.position[0], vertex.position[1], vertex.position[2])
            })
            .collect();

        // TODO: flip uvs?
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

        let indeces: Vec<GLushort> = model.indices;

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

    /// generates a vertex array from the struct data
    #[allow(dead_code)]
    pub fn generate_aos(&self, color: glm::Vec4) -> Vec<Vertex> {
        self.positions
            .iter()
            .zip(self.texture_coords.iter())
            .zip(self.normals.iter())
            .map(|data| Vertex {
                position: *data.0 .0,
                color,
                uv_coords: *data.0 .1,
                normal: *data.1,
                tex_index: 0.0,
            })
            .collect()
    }
}

/// reference counted mesh
#[derive(Clone)]
pub struct SharedMesh(Rc<RefCell<Mesh>>);

impl SharedMesh {
    /// creates a new shared mesh from file
    pub(crate) fn from_file(file_name: &str) -> Self {
        Self(Rc::new(RefCell::new(Mesh::new(file_name))))
    }

    /// creates a new shared mesh from existing mesh
    pub(crate) fn from_mesh(mesh: Mesh) -> Self {
        Self(Rc::new(RefCell::new(mesh)))
    }

    /// borrows the stored value immutably
    pub fn borrow(&self) -> Ref<'_, Mesh> {
        self.0.borrow()
    }
}
