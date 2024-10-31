use crate::glm;
use crate::rendering::data::{LightConfig, LightData, UniformBuffer, Vertex};
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
    pub(crate) light_buffer: UniformBuffer,
    pub(crate) matrix_buffer: UniformBuffer,
    batch_basic: Option<ShaderProgram>,
    instance_basic: Option<ShaderProgram>,
    batch_passthrough: Option<ShaderProgram>,
    instance_passthrough: Option<ShaderProgram>,
}

impl ShaderCatalog {
    /// creates a new shader catalog
    pub fn new() -> Self {
        Self {
            light_buffer: UniformBuffer::new(
                size_of::<LightConfig>()
                    + padding::<LightConfig>()
                    + MAX_LIGHT_SRC_COUNT * size_of::<LightData>(),
            ),
            matrix_buffer: UniformBuffer::new(size_of::<glm::Mat4>() * 2),
            batch_basic: None,
            instance_basic: None,
            batch_passthrough: None,
            instance_passthrough: None,
        }
    }

    /// returns a reference to the basic shader for batch renderers
    pub(crate) fn batch_basic(&mut self) -> &ShaderProgram {
        if self.batch_basic.is_none() {
            self.create_batch_basic();
        }
        self.batch_basic.as_ref().unwrap()
    }

    /// returns a reference to the basic shader for instance renderers
    pub(crate) fn instance_basic(&mut self) -> &ShaderProgram {
        if self.instance_basic.is_none() {
            self.create_instance_basic();
        }
        self.instance_basic.as_ref().unwrap()
    }

    /// returns a reference to the passthrough shader for batch renderers
    pub(crate) fn batch_passthrough(&mut self) -> &ShaderProgram {
        if self.batch_passthrough.is_none() {
            self.create_batch_passthrough();
        }
        self.batch_passthrough.as_ref().unwrap()
    }

    /// returns a reference to the passthrough shader for instance renderers
    pub(crate) fn instance_passthrough(&mut self) -> &ShaderProgram {
        if self.instance_passthrough.is_none() {
            self.create_instance_passthrough();
        }
        self.instance_passthrough.as_ref().unwrap()
    }

    /// creates a new basic batch renderer shader
    fn create_batch_basic(&mut self) {
        let program = ShaderProgram::new(BATCH_B_VERT, BATCH_B_FRAG);

        program.add_unif_buffer("light_data", &self.light_buffer, 0);
        program.add_unif_buffer("matrix_block", &self.matrix_buffer, 1);

        self.batch_basic = Some(program);
    }

    /// creates a new basic instance renderer shader
    fn create_instance_basic(&mut self) {
        let program = ShaderProgram::new(INST_B_VERT, INST_B_FRAG);

        program.add_unif_buffer("light_data", &self.light_buffer, 0);
        program.add_unif_buffer("matrix_block", &self.matrix_buffer, 1);

        self.instance_basic = Some(program);
    }

    /// creates a new passthrough batch renderer shader
    fn create_batch_passthrough(&mut self) {
        let program = ShaderProgram::new(BATCH_PT_VERT, BATCH_PT_FRAG);

        program.add_unif_buffer("matrix_block", &self.matrix_buffer, 1);

        self.batch_passthrough = Some(program);
    }

    /// creates a new passthrough instance renderer shader
    fn create_instance_passthrough(&mut self) {
        let program = ShaderProgram::new(INST_PT_VERT, INST_PT_FRAG);

        program.add_unif_buffer("matrix_block", &self.matrix_buffer, 1);

        self.instance_passthrough = Some(program);
    }
}

/// all shader variants for entity rendering
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
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
