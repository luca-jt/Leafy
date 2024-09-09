use crate::utils::file::get_shader_path;
use gl::types::*;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::read_to_string;
use std::ptr;

/// shader program to use to render
pub(crate) struct ShaderProgram {
    pub(crate) id: GLuint,
    uniform_locations: HashMap<String, GLint>,
    attrib_locations: HashMap<String, GLint>,
}

impl ShaderProgram {
    /// creates new shader program
    pub(crate) fn new(vertex_file: &str, fragment_file: &str) -> Self {
        // compile and link shader program
        let vs_file =
            read_to_string(get_shader_path(vertex_file)).expect("could not find vertex shader");
        let fs_file =
            read_to_string(get_shader_path(fragment_file)).expect("could not find fragment shader");
        let vs = compile_shader(vs_file.as_str(), gl::VERTEX_SHADER);
        let fs = compile_shader(fs_file.as_str(), gl::FRAGMENT_SHADER);
        let id = link_program(vs, fs);

        let uniform_locations = HashMap::new();
        let attrib_locations = HashMap::new();

        let c_out_color = CString::new("out_color").unwrap();
        unsafe {
            gl::BindFragDataLocation(id, 0, c_out_color.as_ptr()); // maybe do this generically
        }

        Self {
            id,
            uniform_locations,
            attrib_locations,
        }
    }

    /// sets an uniform location
    pub(crate) fn add_unif_location(&mut self, name: &str) {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let unif = gl::GetUniformLocation(self.id, c_name.as_ptr());
            self.uniform_locations.insert(name.to_string(), unif);
        }
    }

    /// sets an attrib location
    pub(crate) fn add_attr_location(&mut self, name: &str) {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let attr = gl::GetAttribLocation(self.id, c_name.as_ptr());
            self.attrib_locations.insert(name.to_string(), attr);
        }
    }

    /// gets an uniform location
    pub(crate) fn get_unif(&self, name: &str) -> GLint {
        *self
            .uniform_locations
            .get(name)
            .expect("uniform location not found")
    }

    /// gets an attrib location
    pub(crate) fn get_attr(&self, name: &str) -> GLint {
        *self
            .attrib_locations
            .get(name)
            .expect("attribute location not found")
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

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

/// holds all the shader data and loads them if needed
pub struct ShaderCatalog {
    batch_basic: Option<ShaderProgram>,
    instance_basic: Option<ShaderProgram>,
}

impl ShaderCatalog {
    /// creates a new shader catalog
    pub fn new() -> Self {
        Self {
            batch_basic: None,
            instance_basic: None,
        }
    }

    /// returns a reference to the basic shader for the batch renderer
    pub(crate) fn batch_basic(&mut self) -> &ShaderProgram {
        if self.batch_basic.is_none() {
            self.create_batch_basic();
        }
        self.batch_basic.as_ref().unwrap()
    }

    /// returns a reference to the basic shader for the instance renderer
    pub(crate) fn instance_basic(&mut self) -> &ShaderProgram {
        if self.instance_basic.is_none() {
            self.create_instance_basic();
        }
        self.instance_basic.as_ref().unwrap()
    }

    /// creates a new basic batch renderer shader
    fn create_batch_basic(&mut self) {
        let mut program = ShaderProgram::new("batch_vs.glsl", "batch_fs.glsl");

        program.add_unif_location("projection");
        program.add_unif_location("view");
        program.add_unif_location("model");
        program.add_unif_location("tex_sampler");
        program.add_unif_location("shadow_map");
        program.add_unif_location("light_pos");
        program.add_unif_location("light_matrix");

        program.add_attr_location("position");
        program.add_attr_location("color");
        program.add_attr_location("uv");
        program.add_attr_location("normal");
        program.add_attr_location("tex_idx");

        self.batch_basic = Some(program);
    }

    /// creates a new basic instance renderer shader
    fn create_instance_basic(&mut self) {
        let mut program = ShaderProgram::new("instance_vs.glsl", "instance_fs.glsl");

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
        program.add_attr_location("scale");

        self.instance_basic = Some(program);
    }
}
