use crate::rendering::shader::ShaderProgram;
use crate::utils::constants::{MIN_WIN_HEIGHT, MIN_WIN_WIDTH};
use crate::utils::file::get_texture_path;
use gl::types::*;
use nalgebra_glm as glm;
use stb_image::image::{Image, LoadResult};
use std::collections::HashMap;
use std::ptr;

/// loads an opengl texture
pub fn load_texture(file_name: &str) -> GLuint {
    let mut tex_id = 0;

    let texture: Image<u8>;
    match stb_image::image::load_with_depth(get_texture_path(file_name), 4, false) {
        LoadResult::ImageU8(im) => {
            texture = im;
        }
        _ => {
            panic!("error reading texture")
        }
    }

    // generate gl texture
    unsafe {
        gl::GenTextures(1, &mut tex_id);
        gl::BindTexture(gl::TEXTURE_2D, tex_id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as GLint,
            texture.width as GLint,
            texture.height as GLint,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            texture.data.as_ptr() as *const GLvoid,
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    tex_id
}

#[derive(Default, Clone, Debug)]
#[repr(C)]
/// data for a single vertex
pub struct Vertex {
    pub position: glm::Vec3,
    pub color: glm::Vec4,
    pub uv_coords: glm::Vec2,
    pub normal: glm::Vec3,
    pub tex_index: GLfloat,
}

/// holds the texture ID's for the App
pub struct TextureMap {
    textures: HashMap<String, GLuint>,
}

impl TextureMap {
    /// creates a new texture map
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    /// adds a texture from file
    pub fn add_texture(&mut self, name: &str, file: &str) {
        self.textures.insert(name.to_string(), load_texture(file));
    }

    /// deletes a stored texture
    pub fn delete_texture(&mut self, name: &str) {
        let deleted = self.textures.remove(name).expect("texture not stored");
        unsafe {
            gl::DeleteTextures(1, &deleted);
        }
    }

    /// yields a texture id for given name
    pub fn get_tex_id(&self, name: &str) -> GLuint {
        *self.textures.get(name).expect("texture not in the map")
    }
}

impl Drop for TextureMap {
    fn drop(&mut self) {
        unsafe {
            for (_, texture) in self.textures.iter() {
                gl::DeleteTextures(1, texture);
            }
        }
    }
}

/// stores the current camera config for 3D rendering
pub struct PerspectiveCamera {
    pub projection: glm::Mat4,
    pub view: glm::Mat4,
    pub model: glm::Mat4,
    pub light_src: glm::Vec3,
}

impl PerspectiveCamera {
    /// creates new config with default values
    pub fn new(position: glm::Vec3, focus: glm::Vec3) -> Self {
        let fov = 45.0_f32.to_radians();
        let projection = glm::perspective::<f32>(
            MIN_WIN_WIDTH as f32 / MIN_WIN_HEIGHT as f32,
            fov,
            0.1,
            100.0,
        );
        let up = glm::Vec3::y_axis();
        let view = glm::look_at::<f32>(&position, &focus, &up);
        let model = glm::Mat4::identity();

        Self {
            projection,
            view,
            model,
            light_src: glm::Vec3::new(1.0, 1.0, 1.0),
        }
    }

    /// update the model matrix for a specific object position `(x, y, z)`
    pub fn update_model(&mut self, x: f32, y: f32, z: f32) {
        self.model = glm::translate(&glm::Mat4::identity(), &glm::Vec3::new(x, y, z));
    }

    /// updates the camera for given camera position and focus
    pub fn update_cam(&mut self, position: glm::Vec3, focus: glm::Vec3) {
        self.view = glm::look_at(&position, &focus, &glm::Vec3::y_axis());
    }
}

/// stores the current camera config for 2D rendering
pub struct OrthoCamera {
    pub projection: glm::Mat4,
    pub view: glm::Mat4,
}

impl OrthoCamera {
    /// creates a new orthographic camera
    pub fn new(left: f32, right: f32, bottom: f32, top: f32) -> Self {
        let position = glm::Vec3::new(0.0, 0.0, -1.0);

        Self {
            projection: glm::ortho(left, right, bottom, top, -1.0, 1.0),
            view: glm::look_at(&position, &glm::Vec3::zeros(), &glm::Vec3::y_axis()),
        }
    }

    /// creates a new orthographic camera from a size: `(-size, size, -size, size)`
    pub fn from_size(size: f32) -> Self {
        let position = glm::Vec3::new(0.0, 0.0, -1.0);

        Self {
            projection: glm::ortho(-size, size, -size, size, -1.0, 1.0),
            view: glm::look_at(&position, &glm::Vec3::zeros(), &glm::Vec3::y_axis()),
        }
    }

    /// updates the camera for given camera position and focus
    pub fn update_cam(&mut self, position: glm::Vec3, focus: glm::Vec3) {
        self.view = glm::look_at(&position, &focus, &glm::Vec3::y_axis());
    }
}

/// shadow map used for rendering
pub struct ShadowMap {
    dbo: GLuint,
    shadow_map: GLuint,
    width: GLsizei,
    height: GLsizei,
    pub light_matrix: glm::Mat4,
    light_src: glm::Vec3,
    program: ShaderProgram,
    tmp_viewport: [GLint; 4],
    pub depth_buffer_cleared: bool,
}

impl ShadowMap {
    /// creates a new shadow map with given size
    pub fn new(width: GLsizei, height: GLsizei, light_src: glm::Vec3) -> Self {
        let mut dbo = 0;
        let mut shadow_map = 0;
        let mut program = ShaderProgram::new("shadow_vs.glsl", "shadow_fs.glsl");

        unsafe {
            program.add_attr_location("position");
            program.add_attr_location("offset");
            program.add_unif_location("light_matrix");
            program.add_unif_location("model");

            gl::GenFramebuffers(1, &mut dbo);
            gl::GenTextures(1, &mut shadow_map);

            gl::BindTexture(gl::TEXTURE_2D, shadow_map);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT as GLint,
                width,
                height,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                ptr::null(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);

            gl::BindFramebuffer(gl::FRAMEBUFFER, dbo);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                shadow_map,
                0,
            );
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Self {
            dbo,
            shadow_map,
            width,
            height,
            light_matrix: glm::ortho(-50.0, 50.0, -50.0, 50.0, 0.1, 100.0)
                * glm::look_at(&light_src, &glm::Vec3::zeros(), &glm::Vec3::y_axis()),
            light_src,
            program,
            tmp_viewport: [0; 4],
            depth_buffer_cleared: false,
        }
    }

    /// clears the depth buffer bit of the shadow map if not already done
    pub fn try_clear_depth(&mut self) {
        if !self.depth_buffer_cleared {
            unsafe {
                gl::Clear(gl::DEPTH_BUFFER_BIT);
            }
            self.depth_buffer_cleared = true;
        }
    }

    /// bind the depth buffer for writing
    pub fn bind_writing(&mut self, camera: &PerspectiveCamera) {
        unsafe {
            gl::GetIntegerv(gl::VIEWPORT, &mut self.tmp_viewport[0]);
            gl::Viewport(0, 0, self.width, self.height);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.dbo);
            gl::UseProgram(self.program.id);
            gl::UniformMatrix4fv(
                self.program.get_unif("light_matrix"),
                1,
                gl::FALSE,
                &self.light_matrix[0],
            );
            gl::UniformMatrix4fv(
                self.program.get_unif("model"),
                1,
                gl::FALSE,
                &camera.model[0],
            );
        }
    }

    pub fn unbind_writing(&self) {
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
            gl::Viewport(
                self.tmp_viewport[0],
                self.tmp_viewport[1],
                self.tmp_viewport[2] as GLsizei,
                self.tmp_viewport[3] as GLsizei,
            );
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    /// bind the shadow map for reading
    pub unsafe fn bind_reading(&self, texture_unit: GLuint) {
        gl::BindTextureUnit(texture_unit, self.shadow_map);
    }
}

impl Drop for ShadowMap {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.shadow_map);
            gl::DeleteFramebuffers(1, &self.dbo);
        }
    }
}
