use crate::glm;
use crate::rendering::data::{LightConfig, LightData, UniformBuffer, Vertex};
use crate::systems::rendering_system::{RendererArch, ShaderSpec};
use crate::utils::constants::MAX_LIGHT_SRC_COUNT;
use crate::utils::file::*;
use crate::utils::tools::padding;
use gl::types::*;
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
                std::str::from_utf8(&buf).expect("ShaderInfoLog not valid utf8")
            );
        }
    }
    shader
}

/// links a gl shader program
fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        gl::DetachShader(program, fs);
        gl::DetachShader(program, vs);
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);

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
                std::str::from_utf8(&buf).expect("ProgramInfoLog not valid utf8")
            );
        }
        program
    }
}

/// shader program to use to render
pub(crate) struct ShaderProgram {
    pub(crate) id: GLuint,
}

impl ShaderProgram {
    /// creates new shader program
    pub(crate) fn new(vertex_file: &str, fragment_file: &str) -> Self {
        // compile and link shader program
        let vs = compile_shader(vertex_file, gl::VERTEX_SHADER);
        let fs = compile_shader(fragment_file, gl::FRAGMENT_SHADER);
        let id = link_program(vs, fs);

        let c_out_color = CString::new("out_color").unwrap();
        unsafe {
            gl::BindFragDataLocation(id, 0, c_out_color.as_ptr()); // maybe do this generically
        }
        log::debug!("compiled shader: {:?}", id);

        Self { id }
    }

    /// binds the shader program
    pub(crate) fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
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
        log::debug!("deleted shader: {:?}", self.id);
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

/// holds all the shader data and loads them if needed
pub struct ShaderCatalog {
    batch_basic: ShaderProgram,
    instance_basic: ShaderProgram,
    batch_passthrough: ShaderProgram,
    instance_passthrough: ShaderProgram,
    batch_shadow: ShaderProgram,
    instance_shadow: ShaderProgram,
    pub(crate) light_buffer: UniformBuffer,
    pub(crate) matrix_buffer: UniformBuffer,
}

impl ShaderCatalog {
    /// creates a new shader catalog
    pub fn new() -> Self {
        let light_buffer = UniformBuffer::new(
            size_of::<LightConfig>()
                + padding::<LightConfig>()
                + MAX_LIGHT_SRC_COUNT * size_of::<LightData>(),
        );
        let matrix_buffer = UniformBuffer::new(size_of::<glm::Mat4>() * 2 + size_of::<glm::Vec4>());

        Self {
            batch_basic: Self::create_batch_basic(&light_buffer, &matrix_buffer),
            instance_basic: Self::create_instance_basic(&light_buffer, &matrix_buffer),
            batch_passthrough: Self::create_batch_passthrough(&matrix_buffer),
            instance_passthrough: Self::create_instance_passthrough(&matrix_buffer),
            batch_shadow: Self::create_batch_shadow(),
            instance_shadow: Self::create_instance_shadow(),
            light_buffer,
            matrix_buffer,
        }
    }

    /// calls ``gl::UseProgram`` for the spec's corresponding shader
    pub(crate) fn use_shader(&self, shader_spec: ShaderSpec) {
        match shader_spec.arch {
            RendererArch::Batch => match shader_spec.shader_type {
                ShaderType::Passthrough => self.batch_passthrough.use_program(),
                ShaderType::Basic => self.batch_basic.use_program(),
            },
            RendererArch::Instance => match shader_spec.shader_type {
                ShaderType::Passthrough => self.instance_passthrough.use_program(),
                ShaderType::Basic => self.instance_basic.use_program(),
            },
        }
    }

    /// uses the corresponding shadow shader for the given renderer architecture
    pub(crate) fn use_shadow_shader(&self, arch: RendererArch) {
        match arch {
            RendererArch::Batch => self.batch_shadow.use_program(),
            RendererArch::Instance => self.instance_shadow.use_program(),
        }
    }

    /// creates a new basic batch renderer shader
    fn create_batch_basic(
        light_buffer: &UniformBuffer,
        matrix_buffer: &UniformBuffer,
    ) -> ShaderProgram {
        let program = ShaderProgram::new(BATCH_B_VERT, BATCH_B_FRAG);

        program.add_unif_buffer("light_data", light_buffer, 0);
        program.add_unif_buffer("matrix_block", matrix_buffer, 1);

        program
    }

    /// creates a new basic instance renderer shader
    fn create_instance_basic(
        light_buffer: &UniformBuffer,
        matrix_buffer: &UniformBuffer,
    ) -> ShaderProgram {
        let program = ShaderProgram::new(INSTANCE_B_VERT, INSTANCE_B_FRAG);

        program.add_unif_buffer("light_data", light_buffer, 0);
        program.add_unif_buffer("matrix_block", matrix_buffer, 1);

        program
    }

    /// creates a new passthrough batch renderer shader
    fn create_batch_passthrough(matrix_buffer: &UniformBuffer) -> ShaderProgram {
        let program = ShaderProgram::new(BATCH_PT_VERT, BATCH_PT_FRAG);

        program.add_unif_buffer("matrix_block", matrix_buffer, 1);

        program
    }

    /// creates a new passthrough instance renderer shader
    fn create_instance_passthrough(matrix_buffer: &UniformBuffer) -> ShaderProgram {
        let program = ShaderProgram::new(INSTANCE_PT_VERT, INSTANCE_PT_FRAG);

        program.add_unif_buffer("matrix_block", matrix_buffer, 1);

        program
    }

    /// creates a new shadow rendering shader for the batch renderer
    fn create_batch_shadow() -> ShaderProgram {
        ShaderProgram::new(BATCH_SHADOW_VERT, BATCH_SHADOW_FRAG)
    }

    /// creates a new shadow rendering shader for the instance renderer
    fn create_instance_shadow() -> ShaderProgram {
        ShaderProgram::new(INSTANCE_SHADOW_VERT, INSTANCE_SHADOW_FRAG)
    }
}

/// all shader variants for entity rendering
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub(crate) enum ShaderType {
    Passthrough,
    Basic,
}

/// binds all necessary vertex attrib pointers for the batch renderer depending on the program
pub(crate) unsafe fn bind_batch_attribs(shader_type: ShaderType) {
    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(
        0,
        3,
        gl::FLOAT,
        gl::FALSE as GLboolean,
        size_of::<Vertex>() as GLsizei,
        mem::offset_of!(Vertex, position) as *const GLvoid,
    );

    gl::EnableVertexAttribArray(1);
    gl::VertexAttribPointer(
        1,
        4,
        gl::FLOAT,
        gl::FALSE as GLboolean,
        size_of::<Vertex>() as GLsizei,
        mem::offset_of!(Vertex, color) as *const GLvoid,
    );

    gl::EnableVertexAttribArray(2);
    gl::VertexAttribPointer(
        2,
        2,
        gl::FLOAT,
        gl::FALSE as GLboolean,
        size_of::<Vertex>() as GLsizei,
        mem::offset_of!(Vertex, uv_coords) as *const GLvoid,
    );

    if shader_type != ShaderType::Passthrough {
        gl::EnableVertexAttribArray(3);
        gl::VertexAttribPointer(
            3,
            3,
            gl::FLOAT,
            gl::FALSE as GLboolean,
            size_of::<Vertex>() as GLsizei,
            mem::offset_of!(Vertex, normal) as *const GLvoid,
        );
    }

    gl::EnableVertexAttribArray(4);
    gl::VertexAttribPointer(
        4,
        1,
        gl::FLOAT,
        gl::FALSE as GLboolean,
        size_of::<Vertex>() as GLsizei,
        mem::offset_of!(Vertex, tex_index) as *const GLvoid,
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
