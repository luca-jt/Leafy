use super::data::{PerspectiveCamera, ShadowMap};
use super::shader::ShaderProgram;
use crate::ecs::component::{Color32, Position, Scale};
use crate::glm;
use crate::rendering::mesh::Mesh;
use crate::utils::tools::SharedPtr;
use gl::types::*;
use std::ptr;

/// instance renderer for the 3D rendering option
pub(crate) struct InstanceRenderer {
    vao: GLuint,
    pbo: GLuint,
    tbo: GLuint,
    nbo: GLuint,
    obo: GLuint,
    sbo: GLuint,
    ibo: GLuint,
    white_texture: GLuint,
    index_count: GLsizei,
    shared_mesh: SharedPtr<Mesh>,
    positions: Vec<glm::Vec3>,
    scales: Vec<GLfloat>,
    pos_idx: usize,
    pub(crate) color: Color32,
    pub(crate) tex_id: GLuint,
    num_instances: usize,
}

impl InstanceRenderer {
    /// creates a new instance renderer
    pub(crate) fn new(
        shared_mesh: SharedPtr<Mesh>,
        num_instances: usize,
        program: &ShaderProgram,
    ) -> Self {
        let mesh = shared_mesh.clone();
        let mesh = mesh.borrow();

        let mut vao = 0; // vertex array
        let mut pbo = 0; // positions
        let mut tbo = 0; // uv
        let mut nbo = 0; // normals
        let mut obo = 0; // offsets
        let mut sbo = 0; // scales
        let mut ibo = 0; // indeces
        let mut white_texture = 0;
        let positions = vec![glm::Vec3::zeros(); num_instances];
        let scales = vec![0.0; num_instances];

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

            // offset buffer
            gl::CreateBuffers(1, &mut obo);
            gl::BindBuffer(gl::ARRAY_BUFFER, obo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (num_instances * size_of::<glm::Vec3>()) as GLsizeiptr,
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

            // scale buffer
            gl::CreateBuffers(1, &mut sbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, sbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (num_instances * size_of::<GLfloat>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            gl::EnableVertexAttribArray(program.get_attr("scale") as GLuint);
            gl::VertexAttribPointer(
                program.get_attr("scale") as GLuint,
                1,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );
            gl::VertexAttribDivisor(program.get_attr("scale") as GLuint, 1);

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
            obo,
            sbo,
            ibo,
            white_texture,
            index_count: 0,
            shared_mesh,
            positions,
            scales,
            pos_idx: 0,
            color: Color32::WHITE,
            tex_id: white_texture,
            num_instances,
        }
    }

    /// resizes the internal offset buffer to the specified number of elements (erases all positions added prior to this call)
    pub(crate) fn resize_buffer(&mut self, size: usize) {
        self.num_instances += size;
        self.positions.reserve_exact(size);
        self.scales.reserve_exact(size);
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.obo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.num_instances * size_of::<glm::Vec3>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, self.sbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.num_instances * size_of::<GLfloat>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
        }
    }

    /// adds a position where the mesh shall be rendered
    pub(crate) fn add_position(&mut self, position: &Position, scale: &Scale) {
        if self.pos_idx == self.num_instances {
            panic!("Attempt to draw too many Instances");
        }
        self.positions[self.pos_idx] = *position.data();
        self.scales[self.pos_idx] = scale.0;
        self.index_count += self.shared_mesh.borrow().num_indeces() as GLsizei;
        self.pos_idx += 1;
    }

    /// end position input, copy all the added positions to the gpu
    pub(crate) fn confirm_positions(&self) {
        unsafe {
            // dynamically copy the updated postion data
            let positions_size: GLsizeiptr = (self.pos_idx * size_of::<glm::Vec3>()) as GLsizeiptr;
            gl::BindBuffer(gl::ARRAY_BUFFER, self.obo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                positions_size,
                self.positions[0].as_ptr() as *const GLvoid,
            );
            // dynamically copy the updated scale data
            let scales_size: GLsizeiptr = (self.pos_idx * size_of::<GLfloat>()) as GLsizeiptr;
            gl::BindBuffer(gl::ARRAY_BUFFER, self.sbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                scales_size,
                self.scales.as_ptr() as *const GLvoid,
            );
        }
    }

    /// renders to the shadow map
    pub(crate) fn render_shadows(&self) {
        unsafe {
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
        camera: &PerspectiveCamera,
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
                &camera.projection[0],
            );
            gl::UniformMatrix4fv(program.get_unif("view"), 1, gl::FALSE, &camera.view[0]);
            gl::UniformMatrix4fv(program.get_unif("model"), 1, gl::FALSE, &camera.model[0]);
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
            gl::DeleteBuffers(1, &self.obo);
            gl::DeleteBuffers(1, &self.sbo);
            gl::DeleteBuffers(1, &self.ibo);
            gl::DeleteTextures(1, &self.white_texture);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
