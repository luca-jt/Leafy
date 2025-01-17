use super::data::{ShadowMap, Vertex};
use super::shader::{bind_batch_attribs, ShaderType};
use crate::ecs::component::utils::Color32;
use crate::glm;
use crate::rendering::mesh::Mesh;
use crate::utils::constants::{MAX_LIGHT_SRC_COUNT, MAX_TEXTURE_COUNT};
use crate::utils::tools::mult_mat4_vec3;
use gl::types::*;
use std::collections::HashSet;
use std::ptr;

/// batch renderer for the 3D rendering option
pub(crate) struct BatchRenderer {
    batches: Vec<Batch>,
    used_batch_indices: HashSet<usize>,
    white_texture: GLuint,
    samplers: [GLint; MAX_TEXTURE_COUNT - MAX_LIGHT_SRC_COUNT],
    shadow_samplers: [GLint; MAX_LIGHT_SRC_COUNT],
}

impl BatchRenderer {
    /// creates a new batch renderer
    pub(crate) fn new(mesh: &Mesh, shader_type: ShaderType) -> Self {
        let mut white_texture = 0;

        unsafe {
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
        }
        // TEXTURE SAMPLERS
        let mut samplers: [GLint; MAX_TEXTURE_COUNT - MAX_LIGHT_SRC_COUNT] =
            [0; MAX_TEXTURE_COUNT - MAX_LIGHT_SRC_COUNT];
        for (i, sampler) in samplers.iter_mut().enumerate() {
            *sampler = (i + MAX_LIGHT_SRC_COUNT) as GLint;
        }
        // SHADOW SAMPLERS
        let mut shadow_samplers: [GLint; MAX_LIGHT_SRC_COUNT] = [0; MAX_LIGHT_SRC_COUNT];
        for (i, sampler) in shadow_samplers.iter_mut().enumerate() {
            *sampler = i as GLint;
        }

        Self {
            batches: vec![Batch::new(mesh, shader_type)],
            used_batch_indices: HashSet::new(),
            white_texture,
            samplers,
            shadow_samplers,
        }
    }

    /// confirms data for all render batches and copys it to the GPU
    pub(crate) fn confirm_data(&self) {
        for batch in self.batches.iter() {
            batch.confirm_data();
        }
    }

    /// renders to the shadow map
    pub(crate) fn render_shadows(&self) {
        unsafe {
            // bind uniforms
            gl::Uniform1iv(
                6,
                (MAX_TEXTURE_COUNT - MAX_LIGHT_SRC_COUNT) as GLsizei,
                &self.samplers[0],
            );
            // bind white texture
            gl::BindTextureUnit(MAX_LIGHT_SRC_COUNT as GLuint, self.white_texture);
        }
        for batch in self.batches.iter() {
            batch.render_shadows();
        }
    }

    /// send data to GPU and reset
    pub(crate) fn flush(
        &self,
        shadow_maps: &[&ShadowMap],
        shader_type: ShaderType,
        transparent: bool,
    ) {
        unsafe {
            // bind uniforms
            if shader_type != ShaderType::Passthrough {
                gl::Uniform1i(0, shadow_maps.len() as GLsizei);
                gl::Uniform1iv(2, MAX_LIGHT_SRC_COUNT as GLsizei, &self.shadow_samplers[0]);
            }
            gl::Uniform1i(1, transparent as GLint);
            gl::Uniform1iv(
                7,
                (MAX_TEXTURE_COUNT - MAX_LIGHT_SRC_COUNT) as GLsizei,
                &self.samplers[0],
            );
            // bind textures
            for (i, shadow_map) in shadow_maps.iter().enumerate() {
                shadow_map.bind_reading(i as GLuint);
            }
            gl::BindTextureUnit(MAX_LIGHT_SRC_COUNT as GLuint, self.white_texture);
        }
        for batch in self.batches.iter() {
            batch.flush();
        }
    }

    /// end the rendering process and reset the renderer to the initial state
    pub(crate) fn reset(&mut self) {
        for batch in self.batches.iter_mut() {
            batch.reset();
        }
    }

    /// clean up unused batches
    pub(crate) fn clean_batches(&mut self) {
        for index in 0..self.batches.len() {
            if !self.used_batch_indices.contains(&index) {
                self.batches.remove(index);
            }
        }
        self.used_batch_indices.clear();
    }

    /// draws a mesh with a texture
    pub(crate) fn draw_tex_mesh(
        &mut self,
        trafo: &glm::Mat4,
        tex_id: GLuint,
        mesh: &Mesh,
        shader_type: ShaderType,
    ) {
        for (i, batch) in self.batches.iter_mut().enumerate() {
            if batch.try_add_tex_mesh(trafo, tex_id, mesh) {
                self.used_batch_indices.insert(i);
                return;
            }
        }
        // add new batch
        let mut new_batch = Batch::new(mesh, shader_type);
        let res = new_batch.try_add_tex_mesh(trafo, tex_id, mesh);
        debug_assert!(res);
        self.used_batch_indices.insert(self.batches.len());
        self.batches.push(new_batch);
    }

    /// draws a mesh with a color
    pub(crate) fn draw_color_mesh(&mut self, trafo: &glm::Mat4, color: Color32, mesh: &Mesh) {
        self.used_batch_indices.insert(0);
        self.batches
            .first_mut()
            .unwrap()
            .add_color_mesh(trafo, color, mesh);
    }
}

impl Drop for BatchRenderer {
    fn drop(&mut self) {
        log::debug!("dropped batch renderer");
        unsafe { gl::DeleteTextures(1, &self.white_texture) };
    }
}

/// a single render batch
struct Batch {
    vao: GLuint,
    vbo: GLuint,
    ibo: GLuint,
    index_count: GLsizei,
    obj_buffer: Vec<Vertex>,
    obj_buffer_ptr: usize,
    all_tex_ids: Vec<GLuint>,
    max_num_meshes: usize,
}

impl Batch {
    /// creates a new batch with default size
    fn new(mesh: &Mesh, shader_type: ShaderType) -> Self {
        let max_num_meshes: usize = 10;
        // init the data ids
        let obj_buffer: Vec<Vertex> = vec![Vertex::default(); mesh.num_vertices() * max_num_meshes];
        let mut vao = 0;
        let mut vbo = 0;
        let mut ibo = 0;

        unsafe {
            // GENERATE BUFFERS
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::CreateBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_vertices() * max_num_meshes * size_of::<Vertex>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            // BIND ATTRIB POINTERS
            bind_batch_attribs(shader_type);

            // INDECES
            let mut indices: Vec<GLuint> = vec![0; mesh.num_indices() * max_num_meshes];
            for i in 0..mesh.num_indices() * max_num_meshes {
                indices[i] = mesh.indices[i % mesh.num_indices()]
                    + mesh.num_vertices() as GLuint * (i as GLuint / mesh.num_indices() as GLuint);
            }
            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mesh.num_indices() * max_num_meshes * size_of::<GLuint>()) as GLsizeiptr,
                indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            gl::BindVertexArray(0);
        }
        log::debug!("new batch created");

        Self {
            vao,
            vbo,
            ibo,
            index_count: 0,
            obj_buffer,
            obj_buffer_ptr: 0,
            all_tex_ids: Vec::new(),
            max_num_meshes,
        }
    }

    /// resize the buffer for more mesh data
    fn resize_buffer(&mut self, mesh: &Mesh) {
        let add_size: usize = self.max_num_meshes * 2;
        self.max_num_meshes += add_size;
        self.obj_buffer.reserve_exact(add_size);
        self.obj_buffer
            .extend(vec![Vertex::default(); mesh.num_vertices() * add_size]);
        log::debug!("resized batch to: {:?}", self.max_num_meshes);
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_vertices() * self.max_num_meshes * size_of::<Vertex>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            let mut indices: Vec<GLuint> = vec![0; mesh.num_indices() * self.max_num_meshes];
            for i in 0..mesh.num_indices() * self.max_num_meshes {
                indices[i] = mesh.indices[i % mesh.num_indices()]
                    + mesh.num_vertices() as GLuint * (i as GLuint / mesh.num_indices() as GLuint);
            }
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mesh.num_indices() * self.max_num_meshes * size_of::<GLuint>()) as GLsizeiptr,
                indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
        }
    }

    /// end the render batch
    fn confirm_data(&self) {
        // dynamically copy the the drawn mesh vertex data from object buffer into the vertex buffer on the gpu
        unsafe {
            let vertices_size: GLsizeiptr =
                (self.obj_buffer_ptr * size_of::<Vertex>()) as GLsizeiptr;
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                vertices_size,
                self.obj_buffer.as_ptr() as *const GLvoid,
            );
        }
    }

    /// renders this batch to the shadow map (has to be binded before)
    fn render_shadows(&self) {
        unsafe {
            // bind textures
            for (unit, tex_id) in self.all_tex_ids.iter().enumerate() {
                gl::BindTextureUnit((unit + 1 + MAX_LIGHT_SRC_COUNT) as GLuint, *tex_id);
            }
            // draw the triangles corresponding to the index buffer
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl::BindVertexArray(0);
        }
    }

    /// flushes this batch and renders the stored geometry
    fn flush(&self) {
        unsafe {
            // bind textures
            for (unit, tex_id) in self.all_tex_ids.iter().enumerate() {
                gl::BindTextureUnit((unit + 1 + MAX_LIGHT_SRC_COUNT) as GLuint, *tex_id);
            }
            // draw the triangles corresponding to the index buffer
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl::BindVertexArray(0);
        }
    }

    /// resets the batch to the initial state
    fn reset(&mut self) {
        self.index_count = 0;
        self.obj_buffer_ptr = 0;
    }

    /// adds a mesh with a texture to the batch
    fn try_add_tex_mesh(&mut self, trafo: &glm::Mat4, tex_id: GLuint, mesh: &Mesh) -> bool {
        // determine texture index
        let mut tex_index: GLfloat = -1.0;
        for (i, id) in self.all_tex_ids.iter().enumerate() {
            if *id == tex_id {
                tex_index = (i + 1) as GLfloat;
                break;
            }
        }
        if tex_index == -1.0 {
            if self.all_tex_ids.len() >= MAX_TEXTURE_COUNT - 1 - MAX_LIGHT_SRC_COUNT {
                // start a new batch if out of texture slots
                return false;
            }
            tex_index = (self.all_tex_ids.len() + 1) as GLfloat;
            self.all_tex_ids.push(tex_id);
        }
        if self.index_count as usize >= mesh.num_indices() * self.max_num_meshes {
            // resize current batch if batch size exceeded
            self.resize_buffer(mesh);
        }
        // copy mesh vertex data into the object buffer
        for i in 0..mesh.num_vertices() {
            *self.obj_buffer.get_mut(self.obj_buffer_ptr).unwrap() = Vertex {
                position: mult_mat4_vec3(trafo, &mesh.positions[i]),
                color: glm::vec4(1.0, 1.0, 1.0, 1.0),
                uv_coords: mesh.texture_coords[i],
                normal: glm::mat4_to_mat3(&trafo.try_inverse().unwrap().transpose())
                    * mesh.normals[i],
                tex_index,
            };
            self.obj_buffer_ptr += 1;
        }
        self.index_count += mesh.num_indices() as GLsizei;
        true
    }

    /// adds a mesh with a color to the batch
    fn add_color_mesh(&mut self, trafo: &glm::Mat4, color: Color32, mesh: &Mesh) {
        if self.index_count as usize >= mesh.num_indices() * self.max_num_meshes {
            // resize current batch if batch size exceeded
            self.resize_buffer(mesh);
        }

        let tex_index: GLfloat = 0.0; // white texture

        // copy mesh vertex data into the object buffer
        for i in 0..mesh.num_vertices() {
            *self.obj_buffer.get_mut(self.obj_buffer_ptr).unwrap() = Vertex {
                position: mult_mat4_vec3(trafo, &mesh.positions[i]),
                color: color.to_vec4(),
                uv_coords: mesh.texture_coords[i],
                normal: glm::mat4_to_mat3(&trafo.try_inverse().unwrap().transpose())
                    * mesh.normals[i],
                tex_index,
            };
            self.obj_buffer_ptr += 1;
        }
        self.index_count += mesh.num_indices() as GLsizei;
    }
}

impl Drop for Batch {
    fn drop(&mut self) {
        log::debug!("dropped batch");
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ibo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
