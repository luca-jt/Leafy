use super::data::{PerspectiveCamera, Vertex};
use super::mesh::SharedMesh;
use super::shader::ShaderProgram;
use crate::utils::constants::MAX_TEXTURE_COUNT;
use gl::types::*;
use nalgebra_glm as glm;
use std::{mem, ptr};

/// batch renderer for the 3D rendering option
pub struct BatchRenderer {
    vao: GLuint,
    vbo: GLuint,
    ibo: GLuint,
    white_texture: GLuint,
    index_count: GLsizei,
    obj_buffer: Vec<Vertex>,
    obj_buffer_ptr: usize,
    tex_slots: Vec<GLuint>,
    tex_slot_index: GLuint,
    program: ShaderProgram,
    shared_mesh: SharedMesh,
    max_num_meshes: usize,
    samplers: [i32; MAX_TEXTURE_COUNT],
}

impl BatchRenderer {
    /// creates a new batch renderer
    pub fn new(shared_mesh: SharedMesh, max_num_meshes: usize) -> Self {
        let mesh = shared_mesh.clone();
        let mesh = mesh.borrow();
        // init the data ids
        let obj_buffer: Vec<Vertex> = vec![Vertex::default(); mesh.num_verteces() * max_num_meshes];
        let mut vao = 0;
        let mut vbo = 0;
        let mut ibo = 0;
        let mut program = ShaderProgram::new("batch_vs.glsl", "batch_fs.glsl");
        let mut white_texture = 0;
        let mut tex_slots: Vec<GLuint> = vec![0; MAX_TEXTURE_COUNT];

        unsafe {
            // CREATE SHADER
            program.add_unif_location("projection");
            program.add_unif_location("view");
            program.add_unif_location("model");
            program.add_unif_location("tex_sampler");
            program.add_unif_location("light_pos");

            program.add_attr_location("position");
            program.add_attr_location("color");
            program.add_attr_location("uv");
            program.add_attr_location("normal");
            program.add_attr_location("tex_idx");

            // GENERATE BUFFERS
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::CreateBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_verteces() * max_num_meshes * mem::size_of::<Vertex>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            // BIND ATTRIB POINTERS
            gl::EnableVertexAttribArray(program.get_attr("position") as GLuint);
            gl::VertexAttribPointer(
                program.get_attr("position") as GLuint,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, position) as *const GLvoid,
            );
            gl::EnableVertexAttribArray(program.get_attr("color") as GLuint);
            gl::VertexAttribPointer(
                program.get_attr("color") as GLuint,
                4,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, color) as *const GLvoid,
            );
            gl::EnableVertexAttribArray(program.get_attr("uv") as GLuint);
            gl::VertexAttribPointer(
                program.get_attr("uv") as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, uv_coords) as *const GLvoid,
            );
            gl::EnableVertexAttribArray(program.get_attr("normal") as GLuint);
            gl::VertexAttribPointer(
                program.get_attr("normal") as GLuint,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, normal) as *const GLvoid,
            );
            gl::EnableVertexAttribArray(program.get_attr("tex_idx") as GLuint);
            gl::VertexAttribPointer(
                program.get_attr("tex_idx") as GLuint,
                1,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, tex_index) as *const GLvoid,
            );

            // INDECES
            let mut indeces: Vec<GLushort> = vec![0; mesh.num_indeces() * max_num_meshes];
            for i in 0..mesh.num_indeces() * max_num_meshes {
                indeces[i] = mesh.indeces[i % mesh.num_indeces()]
                    + mesh.num_verteces() as GLushort
                        * (i as GLushort / mesh.num_indeces() as GLushort);
            }
            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mesh.num_indeces() * max_num_meshes * mem::size_of::<GLushort>()) as GLsizeiptr,
                indeces.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // 1x1 WHITE TEXTURE
            gl::GenTextures(1, &mut white_texture);
            gl::BindTexture(gl::TEXTURE_2D, white_texture);
            let white_color_data: Vec<u8> = vec![255, 255, 255];
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as GLint,
                1,
                1,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                white_color_data.as_ptr() as *const GLvoid,
            );
            tex_slots[0] = white_texture;

            gl::BindVertexArray(0);
        }
        // TEXTURE SAMPLERS
        let mut samplers: [GLint; MAX_TEXTURE_COUNT] = [0; MAX_TEXTURE_COUNT];
        for (i, sampler) in samplers.iter_mut().enumerate() {
            *sampler = i as GLint;
        }

        Self {
            vao,
            vbo,
            ibo,
            index_count: 0,
            obj_buffer,
            obj_buffer_ptr: 0,
            tex_slots,
            tex_slot_index: 1,
            program,
            white_texture,
            shared_mesh,
            max_num_meshes,
            samplers,
        }
    }

    /// begin render batch
    pub fn begin_batch(&mut self) {
        self.obj_buffer_ptr = 0;
    }

    /// end render batch
    pub fn end_batch(&mut self) {
        // dynamically copy the the drawn mesh vertex data from object buffer into the vertex buffer on the gpu
        unsafe {
            let verteces_size: GLsizeiptr =
                (self.obj_buffer_ptr * mem::size_of::<Vertex>()) as GLsizeiptr;
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                verteces_size,
                mem::transmute(self.obj_buffer.get(0).unwrap()),
            );
        }
    }

    /// send data to GPU and reset
    pub fn flush(&mut self, camera: &PerspectiveCamera) {
        unsafe {
            // bind shader, textures, uniforms
            gl::UseProgram(self.program.id);
            // bind textures
            for i in 0..self.tex_slot_index {
                gl::BindTextureUnit(i, *self.tex_slots.get(i as usize).unwrap());
            }
            // bind uniforms
            gl::UniformMatrix4fv(
                self.program.get_unif("projection"),
                1,
                gl::FALSE,
                &camera.projection[0],
            );
            gl::UniformMatrix4fv(self.program.get_unif("view"), 1, gl::FALSE, &camera.view[0]);
            gl::UniformMatrix4fv(
                self.program.get_unif("model"),
                1,
                gl::FALSE,
                &camera.model[0],
            );
            gl::Uniform3fv(self.program.get_unif("light_pos"), 1, &camera.light_src[0]);
            gl::Uniform1iv(
                self.program.get_unif("tex_sampler"),
                MAX_TEXTURE_COUNT as GLsizei,
                &self.samplers[0],
            );

            // draw the triangles corresponding to the index buffer
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_SHORT,
                ptr::null(),
            );
            gl::BindVertexArray(0);
        }
        self.index_count = 0;
        self.tex_slot_index = 1;
    }

    /// draws a mesh with a texture
    pub fn draw_tex_mesh(
        &mut self,
        position: glm::Vec3,
        scale: f32,
        tex_id: GLuint,
        camera: &PerspectiveCamera,
    ) {
        let mesh = self.shared_mesh.clone();
        let mesh = mesh.borrow();

        if self.index_count as usize >= mesh.num_indeces() * self.max_num_meshes
            || self.tex_slot_index as usize > MAX_TEXTURE_COUNT
        {
            // start a new batch if batch size exceeded or ran out of texture slots
            self.end_batch();
            self.flush(camera);
            self.begin_batch();
        }

        // determine texture index
        let mut tex_index: GLfloat = 0.0;
        for i in 0..self.tex_slot_index {
            if *self.tex_slots.get(i as usize).unwrap() == tex_id {
                tex_index = i as GLfloat;
                break;
            }
        }
        if tex_index == 0.0 {
            tex_index = self.tex_slot_index as GLfloat;
            *self
                .tex_slots
                .get_mut(self.tex_slot_index as usize)
                .unwrap() = tex_id;
            self.tex_slot_index += 1;
        }

        // copy mesh vertex data into the object buffer
        for i in 0..mesh.num_verteces() {
            *self.obj_buffer.get_mut(self.obj_buffer_ptr).unwrap() = Vertex {
                position: mesh.positions[i] * scale + position,
                color: glm::Vec4::new(1.0, 1.0, 1.0, 1.0),
                uv_coords: mesh.texture_coords[i],
                normal: mesh.normals[i],
                tex_index,
            };
            self.obj_buffer_ptr += 1;
        }
        self.index_count += mesh.num_indeces() as GLsizei;
    }

    /// draws a mesh with a color
    pub fn draw_color_mesh(
        &mut self,
        position: glm::Vec3,
        scale: f32,
        color: glm::Vec4,
        camera: &PerspectiveCamera,
    ) {
        let mesh = self.shared_mesh.clone();
        let mesh = mesh.borrow();

        if self.index_count as usize >= mesh.num_indeces() * self.max_num_meshes {
            // start a new batch if batch size exceeded
            self.end_batch();
            self.flush(camera);
            self.begin_batch();
        }

        let tex_index: GLfloat = 0.0; // white texture

        // copy mesh vertex data into the object buffer
        for i in 0..mesh.num_verteces() {
            *self.obj_buffer.get_mut(self.obj_buffer_ptr).unwrap() = Vertex {
                position: mesh.positions[i] * scale + position,
                color,
                uv_coords: mesh.texture_coords[i],
                normal: mesh.normals[i],
                tex_index,
            };
            self.obj_buffer_ptr += 1;
        }
        self.index_count += mesh.num_indeces() as GLsizei;
    }
}

impl Drop for BatchRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ibo);
            gl::DeleteTextures(1, &self.white_texture);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
