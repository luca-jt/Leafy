use super::data::*;
use super::shader::*;
use crate::internal_prelude::*;
use crate::rendering::mesh::Mesh;
use crate::systems::rendering_system::RenderAttributes;
use std::ptr;

/// instance renderer for the 3D rendering option
pub(crate) struct InstanceRenderer {
    vao: GLuint,
    pbo: GLuint,
    ubo: GLuint,
    nbo: GLuint,
    cbo: GLuint,
    tbo: GLuint,
    mbo: GLuint,
    nmbo: GLuint,
    ibo: GLuint,
    index_count: GLsizei,
    models: Vec<Mat4>,
    normal_matrices: Vec<Mat3>,
    pos_idx: usize,
    attributes: RenderAttributes,
    max_num_instances: usize,
}

impl InstanceRenderer {
    /// creates a new instance renderer
    pub(crate) fn new(mesh: &Mesh, shader_type: ShaderType, attributes: RenderAttributes) -> Self {
        let max_num_instances: usize = 10;

        let mut vao = 0; // vertex array
        let mut pbo = 0; // positions
        let mut ubo = 0; // uv
        let mut nbo = 0; // normals
        let mut cbo = 0; // vertex colors
        let mut tbo = 0; // tangent vectors
        let mut mbo = 0; // models (includes offsets)
        let mut nmbo = 0; // normal matrices
        let mut ibo = 0; // indices
        let models = vec![Mat4::identity(); max_num_instances];
        let normal_matrices = vec![Mat3::identity(); max_num_instances];

        unsafe {
            // GENERATE BUFFERS
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // VERTEX POSITION BUFFER
            gl::GenBuffers(1, &mut pbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, pbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_vertices() * size_of::<Vec3>()) as GLsizeiptr,
                mesh.positions.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            bind_instance_pbo();

            // UV COORDINATE BUFFER
            gl::GenBuffers(1, &mut ubo);
            gl::BindBuffer(gl::ARRAY_BUFFER, ubo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_vertices() * size_of::<Vec2>()) as GLsizeiptr,
                mesh.texture_coords.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            bind_instance_ubo();

            // NORMAL VECTOR BUFFER
            gl::GenBuffers(1, &mut nbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, nbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_vertices() * size_of::<Vec3>()) as GLsizeiptr,
                mesh.normals.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            if shader_type != ShaderType::Passthrough {
                bind_instance_nbo();
            }

            // VERTEX COLOR BUFFER
            gl::GenBuffers(1, &mut cbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, cbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_vertices() * size_of::<Vec4>()) as GLsizeiptr,
                mesh.colors.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            bind_instance_cbo();

            // TANGENT VECTOR BUFFER
            gl::GenBuffers(1, &mut tbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, tbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_vertices() * size_of::<Vec3>()) as GLsizeiptr,
                mesh.tangents.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            if shader_type != ShaderType::Passthrough {
                bind_instance_tbo();
            }

            // MODEL MATRIX BUFFER
            gl::GenBuffers(1, &mut mbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, mbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (max_num_instances * size_of::<Mat4>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            bind_instance_mbo();

            // NORMAL MATRIX BUFFER
            gl::GenBuffers(1, &mut nmbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, nmbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (max_num_instances * size_of::<Mat3>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            if shader_type != ShaderType::Passthrough {
                bind_instance_nmbo();
            }

            // INDECES
            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mesh.num_indices() * size_of::<GLuint>()) as GLsizeiptr,
                mesh.indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            gl::BindVertexArray(0);
        }

        Self {
            vao,
            pbo,
            ubo,
            nbo,
            cbo,
            tbo,
            mbo,
            nmbo,
            ibo,
            index_count: 0,
            models,
            normal_matrices,
            pos_idx: 0,
            attributes,
            max_num_instances,
        }
    }

    /// resizes the internal buffer to hold more instances (erases all positions confirmed prior to this call)
    fn resize_buffer(&mut self) {
        let add_size: usize = self.max_num_instances * 2;
        self.max_num_instances += add_size;

        self.models.reserve_exact(add_size);
        self.models.extend(vec![Mat4::identity(); add_size]);

        self.normal_matrices.reserve_exact(add_size);
        self.normal_matrices
            .extend(vec![Mat3::identity(); add_size]);

        log::debug!(
            "Resized InstanceRenderer to size: {:?}.",
            self.max_num_instances
        );

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.mbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.max_num_instances * size_of::<Mat4>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, self.nmbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.max_num_instances * size_of::<Mat3>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
        }
    }

    /// adds a position where the mesh shall be rendered
    pub(crate) fn add_position(&mut self, trafo: &Mat4, mesh: &Mesh) {
        if self.pos_idx == self.max_num_instances {
            self.resize_buffer();
        }
        self.models[self.pos_idx] = *trafo;
        self.normal_matrices[self.pos_idx] =
            glm::mat4_to_mat3(&trafo.try_inverse().unwrap().transpose());

        self.index_count += mesh.num_indices() as GLsizei;
        self.pos_idx += 1;
    }

    /// end position input, copy all the added positions to the gpu
    pub(crate) fn confirm_positions(&self) {
        unsafe {
            // dynamically copy the updated model data
            let models_size: GLsizeiptr = (self.pos_idx * size_of::<Mat4>()) as GLsizeiptr;
            gl::BindBuffer(gl::ARRAY_BUFFER, self.mbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                models_size,
                self.models.as_ptr() as *const GLvoid,
            );
            let norm_mat_size: GLsizeiptr = (self.pos_idx * size_of::<Mat3>()) as GLsizeiptr;
            gl::BindBuffer(gl::ARRAY_BUFFER, self.nmbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                norm_mat_size,
                self.normal_matrices.as_ptr() as *const GLvoid,
            );
        }
    }

    /// renders to a directional shadow map
    pub(crate) fn render_shadows(&self) {
        unsafe {
            // bind texture
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.attributes.tex_id);
            // bind uniforms
            let color_vec = self.attributes.color.to_vec4();
            gl::Uniform4fv(4, 1, &color_vec[0]);
            gl::Uniform1i(5, 0);
            // draw the instanced triangles corresponding to the index buffer
            gl::BindVertexArray(self.vao);
            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_INT,
                ptr::null(),
                self.pos_idx as GLsizei,
            );
            gl::BindVertexArray(0);
        }
    }

    /// renders to a shadow cube map
    pub(crate) fn render_cube_shadows(&self) {
        unsafe {
            // bind texture
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.attributes.tex_id);
            // bind uniforms
            let color_vec = self.attributes.color.to_vec4();
            gl::Uniform4fv(24, 1, &color_vec[0]);
            gl::Uniform1i(25, 0);
            // draw the instanced triangles corresponding to the index buffer
            gl::BindVertexArray(self.vao);
            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_INT,
                ptr::null(),
                self.pos_idx as GLsizei,
            );
            gl::BindVertexArray(0);
        }
    }

    /// draws the mesh at all the positions specified until the call of this and clears the positions
    pub(crate) fn draw_all<'a>(
        &self,
        dir_shadow_maps: impl Iterator<Item = &'a ShadowMap>,
        cube_shadow_maps: impl Iterator<Item = &'a CubeShadowMap>,
        shader_type: ShaderType,
        transparent: bool,
        white_texture: GLuint,
        is_light_source: bool,
    ) {
        unsafe {
            // bind texture
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.attributes.tex_id);
            for (i, shadow_map) in dir_shadow_maps.enumerate() {
                shadow_map.bind_reading(i as GLuint + 1);
            }
            for (i, shadow_map) in cube_shadow_maps.enumerate() {
                shadow_map.bind_reading((MAX_DIR_LIGHT_MAPS + i) as GLuint + 1);
            }
            gl::ActiveTexture(gl::TEXTURE11);
            gl::BindTexture(gl::TEXTURE_2D, self.attributes.ambient_tex_id);
            gl::ActiveTexture(gl::TEXTURE12);
            gl::BindTexture(gl::TEXTURE_2D, self.attributes.diffuse_tex_id);
            gl::ActiveTexture(gl::TEXTURE13);
            gl::BindTexture(gl::TEXTURE_2D, self.attributes.specular_tex_id);
            gl::ActiveTexture(gl::TEXTURE14);
            gl::BindTexture(gl::TEXTURE_2D, self.attributes.normal_tex_id);

            // bind uniforms
            let color_vec = self.attributes.color.to_vec4();
            gl::Uniform4fv(0, 1, &color_vec[0]);
            gl::Uniform1i(1, 0);
            gl::Uniform1i(2, transparent as GLint);

            if shader_type != ShaderType::Passthrough {
                for i in 0..MAX_DIR_LIGHT_MAPS {
                    gl::Uniform1i(3 + i as GLsizei, i as GLsizei + 1);
                }
                for i in 0..MAX_POINT_LIGHT_MAPS {
                    gl::Uniform1i(8 + i as GLsizei, (MAX_DIR_LIGHT_MAPS + i) as GLsizei + 1);
                }
                gl::Uniform3fv(13, 1, &self.attributes.material_data.ambient_color[0]);
                gl::Uniform3fv(14, 1, &self.attributes.material_data.diffuse_color[0]);
                gl::Uniform3fv(15, 1, &self.attributes.material_data.specular_color[0]);
                gl::Uniform1f(16, self.attributes.material_data.shininess);
                gl::Uniform1i(17, 11);
                gl::Uniform1i(18, 12);
                gl::Uniform1i(19, 13);
                gl::Uniform1i(20, 14);
                let use_normal_map = self.attributes.normal_tex_id != white_texture;
                gl::Uniform1i(21, use_normal_map as GLint);
            } else {
                gl::Uniform1i(22, is_light_source as GLint); // only used in the passthrough shader
            }

            // draw the instanced triangles corresponding to the index buffer
            gl::BindVertexArray(self.vao);
            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_INT,
                ptr::null(),
                self.pos_idx as GLsizei,
            );
            gl::BindVertexArray(0);
        }
    }

    /// resets the renderer to the initial state
    pub(crate) fn reset(&mut self) {
        // reset the positions
        self.index_count = 0;
        self.pos_idx = 0;
    }
}

impl Drop for InstanceRenderer {
    fn drop(&mut self) {
        log::debug!("Dropped InstanceRenderer.");
        unsafe {
            gl::DeleteBuffers(1, &self.pbo);
            gl::DeleteBuffers(1, &self.ubo);
            gl::DeleteBuffers(1, &self.nbo);
            gl::DeleteBuffers(1, &self.cbo);
            gl::DeleteBuffers(1, &self.tbo);
            gl::DeleteBuffers(1, &self.mbo);
            gl::DeleteBuffers(1, &self.nmbo);
            gl::DeleteBuffers(1, &self.ibo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
