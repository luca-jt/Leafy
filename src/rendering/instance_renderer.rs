use super::data::{PerspectiveCamera, ShadowMap};
use super::mesh::SharedMesh;
use super::shader::ShaderProgram;
use crate::ecs::component::Color32;
use gl::types::*;
use nalgebra_glm as glm;
use std::{mem, ptr};

/// instance renderer for the 3D rendering option
pub struct InstanceRenderer {
    vao: GLuint,
    pbo: GLuint,
    tbo: GLuint,
    nbo: GLuint,
    obo: GLuint,
    ibo: GLuint,
    white_texture: GLuint,
    index_count: GLsizei,
    program: ShaderProgram,
    shared_mesh: SharedMesh,
    positions: Vec<glm::Vec3>,
    pos_idx: usize,
    pub color: Color32,
    pub tex_id: GLuint, // TODO
    num_instances: usize,
}

impl InstanceRenderer {
    /// creates a new instance renderer
    pub fn new(shared_mesh: SharedMesh, num_instances: usize, tex_id: Option<GLuint>) -> Self {
        let mesh = shared_mesh.clone();
        let mesh = mesh.borrow();

        let mut vao = 0; // vertex array
        let mut pbo = 0; // positions
        let mut tbo = 0; // uv
        let mut nbo = 0; // normals
        let mut obo = 0; // offsets
        let mut ibo = 0; // indeces
        let mut program = ShaderProgram::new("instance_vs.glsl", "instance_fs.glsl");
        let mut white_texture = 0;
        let positions = vec![glm::Vec3::zeros(); num_instances];

        unsafe {
            // CREATE SHADER
            program.add_unif_location("projection");
            program.add_unif_location("view");
            program.add_unif_location("model");
            program.add_unif_location("tex_sampler");
            program.add_unif_location("shadow_map");
            program.add_unif_location("light_pos");
            program.add_unif_location("color");
            program.add_unif_location("light_matrix");

            program.add_attr_location("position");
            program.add_attr_location("uv");
            program.add_attr_location("normal");
            program.add_attr_location("offset");

            // GENERATE BUFFERS
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // vertex position buffer
            gl::CreateBuffers(1, &mut pbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, pbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.num_verteces() * mem::size_of::<glm::Vec3>()) as GLsizeiptr,
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
                (mesh.num_verteces() * mem::size_of::<glm::Vec2>()) as GLsizeiptr,
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
                (mesh.num_verteces() * mem::size_of::<glm::Vec3>()) as GLsizeiptr,
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

            // offset buffer
            gl::CreateBuffers(1, &mut obo);
            gl::BindBuffer(gl::ARRAY_BUFFER, obo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (num_instances * mem::size_of::<glm::Vec3>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            gl::EnableVertexAttribArray(program.get_attr("offset") as GLuint);
            gl::VertexAttribPointer(
                program.get_attr("offset") as GLuint,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );
            gl::VertexAttribDivisor(program.get_attr("offset") as GLuint, 1);

            // INDECES
            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mesh.num_indeces() * mem::size_of::<GLushort>()) as GLsizeiptr,
                mesh.indeces.as_ptr() as *const GLvoid,
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

            gl::BindVertexArray(0);
        }

        Self {
            vao,
            pbo,
            tbo,
            nbo,
            obo,
            ibo,
            white_texture,
            index_count: 0,
            program,
            shared_mesh,
            positions,
            pos_idx: 0,
            color: Color32::WHITE,
            tex_id: tex_id.unwrap_or(white_texture),
            num_instances,
        }
    }

    /// adds a position where the mesh shall be rendered
    pub fn add_position(&mut self, position: glm::Vec3) {
        if self.pos_idx == self.num_instances {
            panic!("Attempt to draw too many Instances");
            // TODO: resize capacity dynamically?
        }
        self.positions[self.pos_idx] = position;
        self.index_count += self.shared_mesh.borrow().num_indeces() as GLsizei;
        self.pos_idx += 1;
    }

    /// end position input, copy all the added positions to the gpu
    pub fn confirm_positions(&self) {
        unsafe {
            // dynamically copy the updated postion data
            let positions_size: GLsizeiptr =
                (self.pos_idx * mem::size_of::<glm::Vec3>()) as GLsizeiptr;
            gl::BindBuffer(gl::ARRAY_BUFFER, self.obo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                positions_size,
                self.positions[0].as_ptr() as *const GLvoid,
            );
        }
    }

    /// renders to the shadow map
    pub fn render_shadows(&self) {
        unsafe {
            // draw the instanced triangles corresponding to the index buffer
            gl::BindVertexArray(self.vao);
            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_SHORT,
                ptr::null(),
                self.pos_idx as GLsizei,
            );
            gl::BindVertexArray(0);
        }
    }

    /// draws the mesh at all the positions specified until the call of this and clears the positions
    pub fn draw_all(&mut self, camera: &PerspectiveCamera, shadow_map: &ShadowMap) {
        unsafe {
            // bind shader, textures, uniforms
            gl::UseProgram(self.program.id);
            // bind texture
            gl::BindTextureUnit(0, self.tex_id);
            shadow_map.bind_reading(1);
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
            gl::UniformMatrix4fv(
                self.program.get_unif("light_matrix"),
                1,
                gl::FALSE,
                &shadow_map.light_matrix[0],
            );
            gl::Uniform3fv(self.program.get_unif("light_pos"), 1, &camera.light_src[0]);
            gl::Uniform1i(self.program.get_unif("tex_sampler"), 0);
            gl::Uniform1i(self.program.get_unif("shadow_map"), 1);
            gl::Uniform3fv(self.program.get_unif("color"), 1, &self.color.to_vec4()[0]);

            // draw the instanced triangles corresponding to the index buffer
            gl::BindVertexArray(self.vao);
            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_SHORT,
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
            gl::DeleteBuffers(1, &self.obo);
            gl::DeleteBuffers(1, &self.ibo);
            gl::DeleteTextures(1, &self.white_texture);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
