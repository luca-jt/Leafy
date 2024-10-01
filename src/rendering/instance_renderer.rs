use super::data::{calc_model_matrix, Camera, ShadowMap};
use super::shader::ShaderProgram;
use crate::ecs::component::{Color32, Orientation, Position, Scale};
use crate::glm;
use crate::rendering::mesh::Mesh;
use gl::types::*;
use std::ptr;

/// instance renderer for the 3D rendering option
pub(crate) struct InstanceRenderer {
    vao: GLuint,
    pbo: GLuint,
    tbo: GLuint,
    nbo: GLuint,
    mbo: GLuint,
    ibo: GLuint,
    white_texture: GLuint,
    index_count: GLsizei,
    models: Vec<glm::Mat4>,
    pos_idx: usize,
    pub(crate) color: Color32,
    pub(crate) tex_id: GLuint,
    num_instances: usize,
}

impl InstanceRenderer {
    /// creates a new instance renderer
    pub(crate) fn new(mesh: &Mesh, num_instances: usize, program: &ShaderProgram) -> Self {
        let mut vao = 0; // vertex array
        let mut pbo = 0; // positions
        let mut tbo = 0; // uv
        let mut nbo = 0; // normals
        let mut mbo = 0; // models (includes offsets)
        let mut ibo = 0; // indeces
        let mut white_texture = 0;
        let models = vec![glm::Mat4::identity(); num_instances];

        unsafe {
            // GENERATE BUFFERS
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // vertex position buffer
            gl::CreateBuffers(1, &mut pbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, pbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_verteces() * size_of::<glm::Vec3>()) as GLsizeiptr,
                mesh.positions.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(program.get_attr("position") as GLuint);
            gl::VertexAttribPointer(
                program.get_attr("position") as GLuint,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );

            // uv coord buffer
            gl::CreateBuffers(1, &mut tbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, tbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_verteces() * size_of::<glm::Vec2>()) as GLsizeiptr,
                mesh.texture_coords.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(program.get_attr("uv") as GLuint);
            gl::VertexAttribPointer(
                program.get_attr("uv") as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );

            // normal vector buffer
            gl::CreateBuffers(1, &mut nbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, nbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_verteces() * size_of::<glm::Vec3>()) as GLsizeiptr,
                mesh.normals.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(program.get_attr("normal") as GLuint);
            gl::VertexAttribPointer(
                program.get_attr("normal") as GLuint,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );

            // model buffer
            gl::CreateBuffers(1, &mut mbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, mbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (num_instances * size_of::<glm::Mat4>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            let pos1 = program.get_attr("model") as GLuint;
            let pos2 = pos1 + 1;
            let pos3 = pos1 + 2;
            let pos4 = pos1 + 3;
            gl::EnableVertexAttribArray(pos1);
            gl::EnableVertexAttribArray(pos2);
            gl::EnableVertexAttribArray(pos3);
            gl::EnableVertexAttribArray(pos4);
            gl::VertexAttribPointer(
                pos1,
                4,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                (size_of::<GLfloat>() * 4 * 4) as GLsizei,
                ptr::null(),
            );
            gl::VertexAttribPointer(
                pos2,
                4,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                (size_of::<GLfloat>() * 4 * 4) as GLsizei,
                (size_of::<GLfloat>() * 4) as *const GLvoid,
            );
            gl::VertexAttribPointer(
                pos3,
                4,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                (size_of::<GLfloat>() * 4 * 4) as GLsizei,
                (size_of::<GLfloat>() * 8) as *const GLvoid,
            );
            gl::VertexAttribPointer(
                pos4,
                4,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                (size_of::<GLfloat>() * 4 * 4) as GLsizei,
                (size_of::<GLfloat>() * 12) as *const GLvoid,
            );
            gl::VertexAttribDivisor(pos1, 1);
            gl::VertexAttribDivisor(pos2, 1);
            gl::VertexAttribDivisor(pos3, 1);
            gl::VertexAttribDivisor(pos4, 1);

            // INDECES
            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mesh.num_indeces() * size_of::<GLuint>()) as GLsizeiptr,
                mesh.indeces.as_ptr() as *const GLvoid,
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

        Self {
            vao,
            pbo,
            tbo,
            nbo,
            mbo,
            ibo,
            white_texture,
            index_count: 0,
            models,
            pos_idx: 0,
            color: Color32::WHITE,
            tex_id: white_texture,
            num_instances,
        }
    }

    /// resizes the internal offset buffer to the specified number of elements (erases all positions added prior to this call)
    pub(crate) fn resize_buffer(&mut self, size: usize) {
        self.num_instances += size;
        self.models.reserve_exact(size);
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.mbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.num_instances * size_of::<glm::Mat4>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
        }
    }

    /// adds a position where the mesh shall be rendered
    pub(crate) fn add_position(
        &mut self,
        position: &Position,
        scale: &Scale,
        orientation: &Orientation,
        mesh: &Mesh,
    ) {
        if self.pos_idx == self.num_instances {
            panic!("Attempt to draw too many Instances");
        }
        self.models[self.pos_idx] = calc_model_matrix(position, scale, orientation);
        self.index_count += mesh.num_indeces() as GLsizei;
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
        }
    }

    /// renders to the shadow map
    pub(crate) fn render_shadows(&self, shadow_map: &ShadowMap) {
        unsafe {
            gl::Uniform1i(shadow_map.program.get_unif("use_input_model"), 1);
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
        &mut self,
        camera: &impl Camera,
        shadow_map: &ShadowMap,
        program: &ShaderProgram,
    ) {
        unsafe {
            // bind shader, textures, uniforms
            gl::UseProgram(program.id);
            // bind texture
            gl::BindTextureUnit(0, self.tex_id);
            shadow_map.bind_reading(1);
            // bind uniforms
            gl::UniformMatrix4fv(
                program.get_unif("projection"),
                1,
                gl::FALSE,
                &camera.projection()[0],
            );
            gl::UniformMatrix4fv(program.get_unif("view"), 1, gl::FALSE, &camera.view()[0]);
            gl::UniformMatrix4fv(
                program.get_unif("light_matrix"),
                1,
                gl::FALSE,
                &shadow_map.light_matrix[0],
            );
            gl::Uniform3fv(program.get_unif("light_pos"), 1, &shadow_map.light_src[0]);
            gl::Uniform1i(program.get_unif("tex_sampler"), 0);
            gl::Uniform1i(program.get_unif("shadow_map"), 1);
            gl::Uniform4fv(program.get_unif("color"), 1, &self.color.to_vec4()[0]);

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
        // reset the positions
        self.index_count = 0;
        self.pos_idx = 0;
    }
}

impl Drop for InstanceRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.pbo);
            gl::DeleteBuffers(1, &self.tbo);
            gl::DeleteBuffers(1, &self.nbo);
            gl::DeleteBuffers(1, &self.mbo);
            gl::DeleteBuffers(1, &self.ibo);
            gl::DeleteTextures(1, &self.white_texture);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
