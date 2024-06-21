use gl::types::*;
use nalgebra_glm as glm;

use super::data::Vertex;
use crate::utils::file::get_model_path;

/// a mesh that can be rendered in gl
pub struct Mesh {
    pub positions: Vec<glm::Vec3>,
    pub texture_coords: Vec<glm::Vec2>,
    pub normals: Vec<glm::Vec3>,
    pub indeces: Vec<GLushort>,
}

impl Mesh {
    /// creates a new Mesh from an obj file
    pub fn new(file_name: &str) -> Self {
        // load scene data from file
        /*let scene = Scene::from_file(
            get_model_path(file_name).as_str(),
            vec![PostProcess::FlipUVs],
        )
            .expect("failed to load mesh file");*/

        let mut positions: Vec<glm::Vec3> = Vec::new();
        let mut texture_coords: Vec<glm::Vec2> = Vec::new();
        let mut normals: Vec<glm::Vec3> = Vec::new();
        let mut indeces: Vec<GLushort> = Vec::new();

        // convert the data into the required format
        /*for russimp_mesh in scene.meshes {
            let mut added_positions: Vec<glm::Vec3> = russimp_mesh
                .vertices
                .iter()
                .map(|v| glm::Vec3::new(v.x, v.y, v.z))
                .collect();
            positions.append(&mut added_positions);

            let mut added_tex_coords: Vec<glm::Vec2> = russimp_mesh
                .texture_coords
                .first()
                .unwrap()
                .clone()
                .unwrap()
                .iter()
                .map(|uv| glm::Vec2::new(uv.x, uv.y))
                .collect();
            texture_coords.append(&mut added_tex_coords);

            let mut added_normals: Vec<glm::Vec3> = russimp_mesh
                .normals
                .iter()
                .map(|n| glm::Vec3::new(n.x, n.y, n.z))
                .collect();
            normals.append(&mut added_normals);

            let mut added_indeces: Vec<GLushort> = russimp_mesh
                .faces
                .iter()
                .map(|face| -> Vec<GLushort> { face.0.iter().map(|i| *i as GLushort).collect() })
                .flatten()
                .collect();
            indeces.append(&mut added_indeces);
        }*/

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
    pub fn generate_aos(&self, color: glm::Vec3) -> Vec<Vertex> {
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
