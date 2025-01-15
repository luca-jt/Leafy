use super::data::ShadowMap;
use super::shader::*;
use crate::ecs::component::utils::Color32;
use crate::glm;
use crate::rendering::mesh::Mesh;
use crate::utils::constants::MAX_LIGHT_SRC_COUNT;
use gl::types::*;
use std::ptr;

/// instance renderer for the 3D rendering option
pub(crate) struct InstanceRenderer {
    vao: GLuint,
    pbo: GLuint,
    tbo: GLuint,
    nbo: GLuint,
    mbo: GLuint,
    nmbo: GLuint,
    ibo: GLuint,
    white_texture: GLuint,
    index_count: GLsizei,
    models: Vec<glm::Mat4>,
    normal_matrices: Vec<glm::Mat3>,
    pos_idx: usize,
    pub(crate) color: Color32,
    pub(crate) tex_id: GLuint,
    max_num_instances: usize,
    shadow_samplers: [GLint; MAX_LIGHT_SRC_COUNT],
}

impl InstanceRenderer {
    /// creates a new instance renderer
    pub(crate) fn new(mesh: &Mesh, shader_type: ShaderType) -> Self {
        let max_num_instances: usize = 10;

        let mut vao = 0; // vertex array
        let mut pbo = 0; // positions
        let mut tbo = 0; // uv
        let mut nbo = 0; // normals
        let mut mbo = 0; // models (includes offsets)
        let mut nmbo = 0; // normal matrices
        let mut ibo = 0; // indices
        let mut white_texture = 0;
        let models = vec![glm::Mat4::identity(); max_num_instances];
        let normal_matrices = vec![glm::Mat3::identity(); max_num_instances];

        unsafe {
            // GENERATE BUFFERS
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // vertex position buffer
            gl::CreateBuffers(1, &mut pbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, pbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_vertices() * size_of::<glm::Vec3>()) as GLsizeiptr,
                mesh.positions.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            bind_instance_pbo();

            // uv coord buffer
            gl::CreateBuffers(1, &mut tbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, tbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_vertices() * size_of::<glm::Vec2>()) as GLsizeiptr,
                mesh.texture_coords.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            bind_instance_tbo();

            // normal vector buffer
            gl::CreateBuffers(1, &mut nbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, nbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_vertices() * size_of::<glm::Vec3>()) as GLsizeiptr,
                mesh.normals.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            if shader_type != ShaderType::Passthrough {
                bind_instance_nbo();
            }

            // model buffer
            gl::CreateBuffers(1, &mut mbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, mbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (max_num_instances * size_of::<glm::Mat4>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            bind_instance_mbo();

            // normal matrix buffer
            gl::CreateBuffers(1, &mut nmbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, nmbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (max_num_instances * size_of::<glm::Mat3>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            bind_instance_nmbo();

            // INDECES
            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mesh.num_indices() * size_of::<GLuint>()) as GLsizeiptr,
                mesh.indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // 1x1 WHITE TEXTURE
            gl::GenTextures(1, &mut white_texture);
            gl::BindTexture(gl::TEXTURE_2D, white_texture);
            let white_color_data: Vec<u8> = vec![255, 255, 255, 255];
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as GLint,
                1,
                1,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                white_color_data.as_ptr() as *const GLvoid,
            );

            gl::BindVertexArray(0);
        }
        // SHADOW SAMPLERS
        let mut shadow_samplers: [GLint; MAX_LIGHT_SRC_COUNT] = [0; MAX_LIGHT_SRC_COUNT];
        for (i, sampler) in shadow_samplers.iter_mut().enumerate() {
            *sampler = i as GLint + 1;
        }

        Self {
            vao,
            pbo,
            tbo,
            nbo,
            mbo,
            nmbo,
            ibo,
            white_texture,
            index_count: 0,
            models,
            normal_matrices,
            pos_idx: 0,
            color: Color32::WHITE,
            tex_id: white_texture,
            max_num_instances,
            shadow_samplers,
        }
    }

    /// resizes the internal buffer to hold more instances (erases all positions confirmed prior to this call)
    fn resize_buffer(&mut self) {
        let add_size: usize = self.max_num_instances * 2;
        self.max_num_instances += add_size;
        self.models.reserve_exact(add_size);
        self.models.extend(vec![glm::Mat4::identity(); add_size]);
        self.normal_matrices.reserve_exact(add_size);
        self.normal_matrices
            .extend(vec![glm::Mat3::identity(); add_size]);
        log::debug!("resized instance renderer to: {:?}", self.max_num_instances);
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.mbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.max_num_instances * size_of::<glm::Mat4>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
        }
    }

    /// adds a position where the mesh shall be rendered
    pub(crate) fn add_position(&mut self, trafo: &glm::Mat4, mesh: &Mesh) {
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
            let models_size: GLsizeiptr = (self.pos_idx * size_of::<glm::Mat4>()) as GLsizeiptr;
            gl::BindBuffer(gl::ARRAY_BUFFER, self.mbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                models_size,
                self.models.as_ptr() as *const GLvoid,
            );
            let norm_mat_size: GLsizeiptr = (self.pos_idx * size_of::<glm::Mat3>()) as GLsizeiptr;
            gl::BindBuffer(gl::ARRAY_BUFFER, self.nmbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                norm_mat_size,
                self.normal_matrices.as_ptr() as *const GLvoid,
            );
        }
    }

    /// renders to the shadow map
    pub(crate) fn render_shadows(&self) {
        unsafe {
            // bind texture
            gl::BindTextureUnit(0, self.tex_id);
            // bind uniforms
            gl::Uniform1i(7, 0);
            let color_vec = self.color.to_vec4();
            gl::Uniform4fv(1, 1, &color_vec[0]);
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
    pub(crate) fn draw_all(
        &self,
        shadow_maps: &[&ShadowMap],
        shader_type: ShaderType,
        transparent: bool,
    ) {
        unsafe {
            // bind texture
            gl::BindTextureUnit(0, self.tex_id);
            for (i, shadow_map) in shadow_maps.iter().enumerate() {
                shadow_map.bind_reading(i as GLuint + 1);
            }
            // bind uniforms
            if shader_type != ShaderType::Passthrough {
                gl::Uniform1i(0, shadow_maps.len() as GLsizei);
                gl::Uniform1iv(4, MAX_LIGHT_SRC_COUNT as GLsizei, &self.shadow_samplers[0]);
            }
            gl::Uniform1i(2, 0);
            gl::Uniform1i(3, transparent as GLint);
            let color_vec = self.color.to_vec4();
            gl::Uniform4fv(1, 1, &color_vec[0]);

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
    pub(crate) fn end_render_passes(&mut self) {
        // reset the positions
        self.index_count = 0;
        self.pos_idx = 0;
    }
}

impl Drop for InstanceRenderer {
    fn drop(&mut self) {
        log::debug!("dropped instance renderer");
        unsafe {
            gl::DeleteBuffers(1, &self.pbo);
            gl::DeleteBuffers(1, &self.tbo);
            gl::DeleteBuffers(1, &self.nbo);
            gl::DeleteBuffers(1, &self.mbo);
            gl::DeleteBuffers(1, &self.nmbo);
            gl::DeleteBuffers(1, &self.ibo);
            gl::DeleteTextures(1, &self.white_texture);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
