use crate::utils::file::get_shader_path;
use gl::types::*;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::read_to_string;
use std::ptr;

/// shader program to use to render
pub struct ShaderProgram {
    pub id: GLuint,
    uniform_locations: HashMap<String, GLint>,
    attrib_locations: HashMap<String, GLint>,
}

impl ShaderProgram {
    /// creates new shader program
    pub fn new(vertex_file: &str, fragment_file: &str) -> Self {
        // compile and link shader program
        let vs_file = read_to_string(get_shader_path(vertex_file)).unwrap();
        let fs_file = read_to_string(get_shader_path(fragment_file)).unwrap();
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
    pub unsafe fn add_unif_location(&mut self, name: &str) {
        let c_name = CString::new(name).unwrap();
        let unif = gl::GetUniformLocation(self.id, c_name.as_ptr());
        self.uniform_locations.insert(name.to_string(), unif);
    }

    /// sets an attrib location
    pub unsafe fn add_attr_location(&mut self, name: &str) {
        let c_name = CString::new(name).unwrap();
        let attr = gl::GetAttribLocation(self.id, c_name.as_ptr());
        self.attrib_locations.insert(name.to_string(), attr);
    }

    /// gets an uniform location
    pub fn get_unif(&self, name: &str) -> GLint {
        *self.uniform_locations.get(name).unwrap()
    }

    /// gets an attrib location
    pub fn get_attr(&self, name: &str) -> GLint {
        *self.attrib_locations.get(name).unwrap()
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
                str::from_utf8(&buf).expect("ShaderInfoLog not valid utf8")
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
                str::from_utf8(&buf).expect("ProgramInfoLog not valid utf8")
            );
        }
        program
    }
}
