use crate::ecs::component::{Density, Scale};
use crate::glm;
use crate::utils::constants::ORIGIN;
use crate::utils::tools::to_vec4;
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
            .map(|vertex| glm::vec3(vertex.position[0], vertex.position[1], vertex.position[2]))
            .collect();

        let texture_coords: Vec<glm::Vec2> = model
            .vertices
            .iter()
            .map(|vertex| glm::vec2(vertex.texture[0], vertex.texture[1]))
            .collect();

        let normals: Vec<glm::Vec3> = model
            .vertices
            .iter()
            .map(|vertex| glm::vec3(vertex.normal[0], vertex.normal[1], vertex.normal[2]))
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
    #[inline]
    pub fn num_verteces(&self) -> usize {
        self.positions.len()
    }

    /// yields the number of indeces per object
    #[inline]
    pub fn num_indeces(&self) -> usize {
        self.indeces.len()
    }

    /// generates the inertia tensor matrix
    pub(crate) fn intertia_tensor(&self, density: &Density, scale: &Scale) -> glm::Mat3 {
        // inertia matrix entries
        let (mut ia, mut ib, mut ic, mut iap, mut ibp, mut icp) =
            (0f32, 0f32, 0f32, 0f32, 0f32, 0f32);
        let mut mass_center = ORIGIN;
        let mut mass = 0f32;
        let scale_matrix = scale.scale_matrix();

        for i in (0..self.indeces.len()).step_by(3) {
            let pos1 = self.positions[self.indeces[i] as usize];
            let pos2 = self.positions[self.indeces[i + 1] as usize];
            let pos3 = self.positions[self.indeces[i + 2] as usize];
            let scaled1 = (scale_matrix * to_vec4(&pos1)).xyz();
            let scaled2 = (scale_matrix * to_vec4(&pos2)).xyz();
            let scaled3 = (scale_matrix * to_vec4(&pos3)).xyz();
            let triangle = (scaled1, scaled2, scaled3);
            let det_jacobi = triangle.0.dot(&triangle.1.cross(&triangle.2));
            let tet_volume = det_jacobi / 6.0;
            let tet_mass = tet_volume * density.0;
            let tet_mass_center = (triangle.0 + triangle.1 + triangle.2) / 4.0;

            ia += det_jacobi * (inertia_moment(&triangle, 1) + inertia_moment(&triangle, 2));
            ib += det_jacobi * (inertia_moment(&triangle, 0) + inertia_moment(&triangle, 2));
            ic += det_jacobi * (inertia_moment(&triangle, 0) + inertia_moment(&triangle, 1));
            iap += det_jacobi * inertia_product(&triangle, 1, 2);
            ibp += det_jacobi * inertia_product(&triangle, 0, 1);
            icp += det_jacobi * inertia_product(&triangle, 0, 2);

            mass_center += tet_mass * tet_mass_center;
            mass += tet_mass;
        }
        mass_center /= mass;
        ia = density.0 * ia / 60.0 - mass * (mass_center.y.sqrt() + mass_center.z.sqrt());
        ib = density.0 * ib / 60.0 - mass * (mass_center.x.sqrt() + mass_center.z.sqrt());
        ic = density.0 * ic / 60.0 - mass * (mass_center.x.sqrt() + mass_center.y.sqrt());
        iap = density.0 * ia / 120.0 - mass * mass_center.y * mass_center.z;
        ibp = density.0 * ia / 120.0 - mass * mass_center.x * mass_center.y;
        icp = density.0 * ia / 120.0 - mass * mass_center.x * mass_center.z;

        glm::Mat3::from_columns(&[
            glm::vec3(ia, -ibp, -icp),
            glm::vec3(-ibp, ib, -iap),
            glm::vec3(-icp, -iap, ic),
        ])
    }
}

/// computes the inertia moment for a given traingle and index
fn inertia_moment(triangle: &(glm::Vec3, glm::Vec3, glm::Vec3), i: usize) -> f32 {
    triangle.0[i].sqrt()
        + triangle.1[i] * triangle.2[i]
        + triangle.1[i].sqrt()
        + triangle.0[i] * triangle.2[i]
        + triangle.2[i].sqrt()
        + triangle.0[i] * triangle.1[i]
}

/// computes the inertia product for a given traingle and indeces
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
