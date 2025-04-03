use crate::internal_prelude::*;
use crate::rendering::data::*;
use crate::rendering::sprite_renderer::SpriteVertex;
use std::ffi::CString;
use std::{mem, ptr};

/// compiles a gl shader
fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        // Create GLSL shaders
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.fill(0);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                std::str::from_utf8(&buf).expect("ShaderInfoLog is not valid UTF-8.")
            );
        }
    }
    shader
}

/// links a gl shader program
fn link_program(vs: GLuint, fs: GLuint, gs: Option<GLuint>) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        if let Some(gs) = gs {
            gl::AttachShader(program, gs);
        }
        gl::LinkProgram(program);

        gl::DetachShader(program, fs);
        gl::DetachShader(program, vs);
        if let Some(gs) = gs {
            gl::DetachShader(program, gs);
        }
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);
        if let Some(gs) = gs {
            gl::DeleteShader(gs);
        }

        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.fill(0);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                std::str::from_utf8(&buf).expect("ProgramInfoLog is not valid UTF-8.")
            );
        }
        program
    }
}

/// shader program to use to render
pub(crate) struct ShaderProgram {
    pub(crate) id: GLuint,
    name: &'static str,
}

impl ShaderProgram {
    /// creates new shader program
    pub(crate) fn new(
        vertex_file: &str,
        fragment_file: &str,
        geometry_file: Option<&str>,
        name: &'static str,
    ) -> Self {
        // compile and link shader program
        let start_time = Instant::now();

        let vs = compile_shader(vertex_file, gl::VERTEX_SHADER);
        let fs = compile_shader(fragment_file, gl::FRAGMENT_SHADER);
        let gs = geometry_file.map(|file| compile_shader(file, gl::GEOMETRY_SHADER));
        let id = link_program(vs, fs, gs);

        let elapsed_time = start_time.elapsed().as_micros() as f64 / 1000.0;

        let c_out_color = CString::new("out_color").unwrap();
        unsafe { gl::BindFragDataLocation(id, 0, c_out_color.as_ptr()) };
        log::debug!("Compiled shader {name:?}: {elapsed_time:.2}ms");

        Self { id, name }
    }

    /// binds the shader program
    pub(crate) fn use_program(&self) {
        unsafe { gl::UseProgram(self.id) };
    }

    /// binds a new uniform buffer
    pub(crate) fn add_unif_buffer(&self, name: &str, buffer: &UniformBuffer, index: GLuint) {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let block_index = gl::GetUniformBlockIndex(self.id, c_name.as_ptr());
            gl::UniformBlockBinding(self.id, block_index, index);
            gl::BindBufferBase(gl::UNIFORM_BUFFER, index, buffer.ubo);
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        log::trace!("Deleted shader: {:?}.", self.name);
        unsafe { gl::DeleteProgram(self.id) };
    }
}

/// holds all the shader data and loads them if needed
pub(crate) struct ShaderCatalog {
    render_shaders: AHashMap<ShaderType, ShaderProgram>,
    pub(crate) shadow: ShaderProgram,
    pub(crate) cube_shadow: ShaderProgram,
    pub(crate) skybox: ShaderProgram,
    pub(crate) screen: ShaderProgram,
    pub(crate) sprite: ShaderProgram,
    pub(crate) light_buffer: UniformBuffer,
    pub(crate) matrix_buffer: UniformBuffer,
    pub(crate) ortho_buffer: UniformBuffer,
}

impl ShaderCatalog {
    /// creates a new shader catalog
    pub(crate) fn new() -> Self {
        let light_buffer = UniformBuffer::new(
            size_of::<LightConfig>()
                + padding::<LightConfig>()
                + MAX_DIR_LIGHT_MAPS * size_of::<DirLightData>()
                + MAX_POINT_LIGHT_COUNT * size_of::<PointLightData>()
                + size_of::<GLint>() * 2,
        );
        let matrix_buffer = UniformBuffer::new(size_of::<Mat4>() * 2 + size_of::<Vec4>());
        let ortho_buffer = UniformBuffer::new(size_of::<Mat4>() * 2);

        let mut render_shaders = AHashMap::new();
        render_shaders.insert(
            ShaderType::Basic,
            Self::create_basic(&light_buffer, &matrix_buffer),
        );
        render_shaders.insert(
            ShaderType::Passthrough,
            Self::create_passthrough(&matrix_buffer),
        );

        Self {
            render_shaders,
            shadow: Self::create_shadow(),
            cube_shadow: Self::create_cube_shadow(),
            skybox: Self::create_skybox(&matrix_buffer),
            screen: Self::create_screen(),
            sprite: Self::create_sprite(&ortho_buffer),
            light_buffer,
            matrix_buffer,
            ortho_buffer,
        }
    }

    /// calls ``gl::UseProgram`` for the shader type's corresponding shader
    pub(crate) fn use_shader(&self, shader_type: &ShaderType) {
        self.render_shaders.get(shader_type).unwrap().use_program();
    }

    /// creates a new sprite shader
    fn create_sprite(ortho_buffer: &UniformBuffer) -> ShaderProgram {
        let program = ShaderProgram::new(SPRITE_VERT, SPRITE_FRAG, None, "Sprite");

        program.add_unif_buffer("ortho_block", ortho_buffer, 2);

        program
    }

    /// creates a new screen texture shader
    fn create_screen() -> ShaderProgram {
        ShaderProgram::new(SCREEN_VERT, SCREEN_FRAG, None, "Screen Texture")
    }

    /// creates a new skybox shader
    fn create_skybox(matrix_buffer: &UniformBuffer) -> ShaderProgram {
        let program = ShaderProgram::new(SKYBOX_VERT, SKYBOX_FRAG, None, "Skybox");

        program.add_unif_buffer("matrix_block", matrix_buffer, 1);

        program
    }

    /// creates a new basic instance renderer shader
    fn create_basic(light_buffer: &UniformBuffer, matrix_buffer: &UniformBuffer) -> ShaderProgram {
        let program = ShaderProgram::new(BASIC_VERT, BASIC_FRAG, None, "Instance Basic");

        program.add_unif_buffer("light_data", light_buffer, 0);
        program.add_unif_buffer("matrix_block", matrix_buffer, 1);

        program
    }

    /// creates a new passthrough instance renderer shader
    fn create_passthrough(matrix_buffer: &UniformBuffer) -> ShaderProgram {
        let program = ShaderProgram::new(
            PASSTHROUGH_VERT,
            PASSTHROUGH_FRAG,
            None,
            "Instance Passthrough",
        );

        program.add_unif_buffer("matrix_block", matrix_buffer, 1);

        program
    }

    /// creates a new shadow rendering shader for the instance renderer
    fn create_shadow() -> ShaderProgram {
        ShaderProgram::new(SHADOW_VERT, SHADOW_FRAG, None, "Instance Shadow")
    }

    /// creates a new cube map shadow rendering shader for the instance renderer
    fn create_cube_shadow() -> ShaderProgram {
        ShaderProgram::new(
            CUBE_SHADOW_VERT,
            CUBE_SHADOW_FRAG,
            Some(CUBE_SHADOW_GEOM),
            "Instance Cube Shadow",
        )
    }
}

/// binds all necessary vertex attrib pointers for the sprite batch renderer
pub(crate) unsafe fn bind_sprite_attribs() {
    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(
        0,
        3,
        gl::FLOAT,
        gl::FALSE as GLboolean,
        size_of::<SpriteVertex>() as GLsizei,
        mem::offset_of!(SpriteVertex, position) as *const GLvoid,
    );

    gl::EnableVertexAttribArray(1);
    gl::VertexAttribPointer(
        1,
        4,
        gl::FLOAT,
        gl::FALSE as GLboolean,
        size_of::<SpriteVertex>() as GLsizei,
        mem::offset_of!(SpriteVertex, color) as *const GLvoid,
    );

    gl::EnableVertexAttribArray(2);
    gl::VertexAttribPointer(
        2,
        2,
        gl::FLOAT,
        gl::FALSE as GLboolean,
        size_of::<SpriteVertex>() as GLsizei,
        mem::offset_of!(SpriteVertex, uv_coords) as *const GLvoid,
    );

    gl::EnableVertexAttribArray(3);
    gl::VertexAttribPointer(
        3,
        1,
        gl::FLOAT,
        gl::FALSE as GLboolean,
        size_of::<SpriteVertex>() as GLsizei,
        mem::offset_of!(SpriteVertex, tex_index) as *const GLvoid,
    );
}

/// binds the pbo attrib pointer for the instance renderer
pub(crate) unsafe fn bind_instance_pbo() {
    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE as GLboolean, 0, ptr::null());
}

/// binds the tbo attrib pointer for the instance renderer
pub(crate) unsafe fn bind_instance_tbo() {
    gl::EnableVertexAttribArray(1);
    gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE as GLboolean, 0, ptr::null());
}

/// binds the nbo attrib pointer for the instance renderer
pub(crate) unsafe fn bind_instance_nbo() {
    gl::EnableVertexAttribArray(2);
    gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE as GLboolean, 0, ptr::null());
}

/// binds the cbo attrib pointer for the instance renderer
pub(crate) unsafe fn bind_instance_cbo() {
    gl::EnableVertexAttribArray(3);
    gl::VertexAttribPointer(3, 4, gl::FLOAT, gl::FALSE as GLboolean, 0, ptr::null());
}

/// binds the mbo attrib pointer for the instance renderer
pub(crate) unsafe fn bind_instance_mbo() {
    let pos1 = 5;
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
}

/// binds the nmbo attrib pointer for the instance renderer
pub(crate) unsafe fn bind_instance_nmbo() {
    let pos1 = 9;
    let pos2 = pos1 + 1;
    let pos3 = pos1 + 2;
    gl::EnableVertexAttribArray(pos1);
    gl::EnableVertexAttribArray(pos2);
    gl::EnableVertexAttribArray(pos3);
    gl::VertexAttribPointer(
        pos1,
        3,
        gl::FLOAT,
        gl::FALSE as GLboolean,
        (size_of::<GLfloat>() * 3 * 3) as GLsizei,
        ptr::null(),
    );
    gl::VertexAttribPointer(
        pos2,
        3,
        gl::FLOAT,
        gl::FALSE as GLboolean,
        (size_of::<GLfloat>() * 3 * 3) as GLsizei,
        (size_of::<GLfloat>() * 3) as *const GLvoid,
    );
    gl::VertexAttribPointer(
        pos3,
        3,
        gl::FLOAT,
        gl::FALSE as GLboolean,
        (size_of::<GLfloat>() * 3 * 3) as GLsizei,
        (size_of::<GLfloat>() * 6) as *const GLvoid,
    );
    gl::VertexAttribDivisor(pos1, 1);
    gl::VertexAttribDivisor(pos2, 1);
    gl::VertexAttribDivisor(pos3, 1);
}
