use crate::ecs::component::*;
use crate::glm;
use crate::rendering::shader::ShaderProgram;
use crate::utils::constants::*;
use crate::utils::file::get_texture_path;
use gl::types::*;
use stb_image::image::{Image, LoadResult};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::Path;
use std::ptr;

/// loads an opengl texture
pub fn load_texture(file_path: impl AsRef<Path>) -> GLuint {
    let mut tex_id = 0;

    let texture: Image<u8>;
    match stb_image::image::load_with_depth(file_path, 4, false) {
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
            gl::RGBA8 as GLint,
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

/// data for a single vertex
#[derive(Default, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub color: glm::Vec4,
    pub uv_coords: glm::Vec2,
    pub normal: glm::Vec3,
    pub tex_index: GLfloat,
}

/// holds the texture ID's for the App
pub(crate) struct TextureMap {
    textures: HashMap<String, GLuint>,
}

impl TextureMap {
    /// creates a new texture map
    pub(crate) fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    /// adds a texture from file
    pub(crate) fn add_texture(&mut self, texture: &Texture) {
        match texture {
            Texture::Ice => {
                self.textures.insert(
                    String::from("ice"),
                    load_texture(get_texture_path("ice.png")),
                );
            }
            Texture::Sand => {
                self.textures.insert(
                    String::from("sand"),
                    load_texture(get_texture_path("sand.png")),
                );
            }
            Texture::Wall => {
                self.textures.insert(
                    String::from("wall"),
                    load_texture(get_texture_path("wall.png")),
                );
            }
            Texture::Custom(path) => {
                self.textures.insert(
                    path.file_name().unwrap().to_str().unwrap().to_string(),
                    load_texture(path),
                );
            }
        }
    }

    /// deletes a stored texture
    pub(crate) fn delete_texture(&mut self, texture: &Texture) {
        let deleted = match texture {
            Texture::Ice => self.textures.remove("ice").expect("texture not stored"),
            Texture::Sand => self.textures.remove("sand").expect("texture not stored"),
            Texture::Wall => self.textures.remove("wall").expect("texture not stored"),
            Texture::Custom(path) => self
                .textures
                .remove(path.file_name().unwrap().to_str().unwrap())
                .expect("texture not stored"),
        };
        unsafe {
            gl::DeleteTextures(1, &deleted);
        }
    }

    /// yields a texture id for given name
    pub(crate) fn get_tex_id(&self, texture: &Texture) -> Option<GLuint> {
        match texture {
            Texture::Ice => self.textures.get("ice").copied(),
            Texture::Sand => self.textures.get("sand").copied(),
            Texture::Wall => self.textures.get("wall").copied(),
            Texture::Custom(path) => self
                .textures
                .get(path.file_name().unwrap().to_str().unwrap())
                .copied(),
        }
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

/// allows for fluent exchange of camera implementation details in rendering
pub(crate) trait Camera {
    /// access to the projection matrix
    fn projection(&self) -> &glm::Mat4;
    /// access to the view matrix
    fn view(&self) -> &glm::Mat4;
}

/// calculate the model matrix for a given position, scale and orientation
pub fn calc_model_matrix(
    position: &Position,
    scale: &Scale,
    orientation: &Orientation,
) -> glm::Mat4 {
    let translate = glm::translate(&glm::Mat4::identity(), position.data());
    let rotate = orientation.rotation_matrix();
    let scaled = scale.scale_matrix();
    translate * rotate * scaled
}

/// stores the current camera config for 3D rendering
pub(crate) struct PerspectiveCamera {
    projection: glm::Mat4,
    view: glm::Mat4,
}

impl PerspectiveCamera {
    /// creates new config with default values
    pub(crate) fn new(position: glm::Vec3, focus: glm::Vec3) -> Self {
        let fov = 45.0_f32.to_radians();
        let projection = glm::perspective::<f32>(
            MIN_WIN_WIDTH as f32 / MIN_WIN_HEIGHT as f32,
            fov,
            0.1,
            100.0,
        );
        let up = glm::Vec3::y_axis();
        let view = glm::look_at::<f32>(&position, &focus, &up);

        Self { projection, view }
    }

    /// update the projection matrix based on a given fov
    pub(crate) fn update_fov(&mut self, fov: f32) {
        self.projection = glm::perspective::<f32>(
            MIN_WIN_WIDTH as f32 / MIN_WIN_HEIGHT as f32,
            fov.to_radians(),
            0.1,
            100.0,
        );
    }

    /// updates the camera for given camera position and focus
    pub(crate) fn update_cam(&mut self, position: &glm::Vec3, focus: &glm::Vec3) {
        self.view = glm::look_at(position, focus, &glm::Vec3::y_axis());
    }
}

impl Camera for PerspectiveCamera {
    fn projection(&self) -> &glm::Mat4 {
        &self.projection
    }

    fn view(&self) -> &glm::Mat4 {
        &self.view
    }
}

/// stores the current camera config for 2D rendering
pub(crate) struct OrthoCamera {
    projection: glm::Mat4,
    view: glm::Mat4,
}

impl OrthoCamera {
    /// creates a new orthographic camera
    pub(crate) fn new(left: f32, right: f32, bottom: f32, top: f32) -> Self {
        let position = -Z_AXIS;

        Self {
            projection: glm::ortho(left, right, bottom, top, -1.0, 1.0),
            view: glm::look_at(&position, &ORIGIN, &glm::Vec3::y_axis()),
        }
    }

    /// creates a new orthographic camera from a size: `(-size, size, -size, size)`
    pub(crate) fn from_size(size: f32) -> Self {
        Self::new(-size, size, -size, size)
    }
}

impl Camera for OrthoCamera {
    fn projection(&self) -> &glm::Mat4 {
        &self.projection
    }

    fn view(&self) -> &glm::Mat4 {
        &self.view
    }
}

/// shadow map used for rendering
pub(crate) struct ShadowMap {
    dbo: GLuint,
    shadow_map: GLuint,
    size: (GLsizei, GLsizei),
    pub(crate) light_matrix: glm::Mat4,
    pub(crate) light_src: glm::Vec3,
    pub(crate) program: ShaderProgram,
    tmp_viewport: [GLint; 4],
}

impl ShadowMap {
    /// creates a new shadow map with given size (width, height)
    pub(crate) fn new(size: (GLsizei, GLsizei), light_src: glm::Vec3) -> Self {
        let mut dbo = 0;
        let mut shadow_map = 0;
        let mut program = ShaderProgram::new("shadow_vs.glsl", "shadow_fs.glsl");

        unsafe {
            program.add_attr_location("position");
            program.add_attr_location("model");

            program.add_unif_location("light_matrix");
            program.add_unif_location("use_input_model");

            gl::GenFramebuffers(1, &mut dbo);
            gl::GenTextures(1, &mut shadow_map);

            gl::BindTexture(gl::TEXTURE_2D, shadow_map);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT as GLint,
                size.0,
                size.1,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                ptr::null(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_BORDER as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_BORDER as GLint,
            );
            let border_color = Color32::WHITE.to_vec4();
            gl::TexParameterfv(
                gl::TEXTURE_2D,
                gl::TEXTURE_BORDER_COLOR,
                border_color.as_ptr(),
            );

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
            size,
            light_matrix: glm::ortho(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0)
                * glm::look_at(&light_src, &ORIGIN, &Y_AXIS),
            light_src,
            program,
            tmp_viewport: [0; 4],
        }
    }

    /// bind the depth buffer for writing
    pub(crate) fn bind_writing(&mut self) {
        unsafe {
            gl::GetIntegerv(gl::VIEWPORT, &mut self.tmp_viewport[0]);
            gl::Viewport(0, 0, self.size.0, self.size.1);
            gl::Scissor(0, 0, self.size.0, self.size.1);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.dbo);
            gl::UseProgram(self.program.id);
            gl::UniformMatrix4fv(
                self.program.get_unif("light_matrix"),
                1,
                gl::FALSE,
                &self.light_matrix[0],
            );
            // clear the depth buffer bit
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }
    }

    /// unbinds the shadow map and restores the regular viewport
    pub(crate) fn unbind_writing(&self) {
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
            gl::Viewport(
                self.tmp_viewport[0],
                self.tmp_viewport[1],
                self.tmp_viewport[2] as GLsizei,
                self.tmp_viewport[3] as GLsizei,
            );
            gl::Scissor(
                self.tmp_viewport[0],
                self.tmp_viewport[1],
                self.tmp_viewport[2] as GLsizei,
                self.tmp_viewport[3] as GLsizei,
            );
        }
    }

    /// bind the shadow map for reading
    pub(crate) unsafe fn bind_reading(&self, texture_unit: GLuint) {
        gl::BindTextureUnit(texture_unit, self.shadow_map);
    }

    /// updates the shadow map according to a new light position
    pub(crate) fn update_light_pos(&mut self, pos: &glm::Vec3) {
        self.light_src = *pos;
        self.light_matrix =
            glm::ortho(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0) * glm::look_at(pos, &ORIGIN, &Y_AXIS);
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

/// one light data block for uniform buffer use
#[repr(C)]
pub(crate) struct LightData {
    pub(crate) light_src: glm::Vec4,
    pub(crate) light_matrix: glm::Mat4,
}

/// uniform buffer wrapper for one array of uniforms
pub(crate) struct UnifBufArray<T> {
    pub(crate) ubo: GLuint,
    size: usize,
    phantom: PhantomData<T>,
}

impl<T> UnifBufArray<T> {
    /// creates a new uniform buffer
    pub(crate) fn new(size: usize) -> Self {
        let mut ubo = 0;
        unsafe {
            gl::GenBuffers(1, &mut ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, ubo);
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                (size * size_of::<T>()) as GLsizeiptr,
                ptr::null(),
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
        Self {
            ubo,
            size,
            phantom: PhantomData,
        }
    }

    /// uploads data to the buffer
    pub(crate) fn upload_data(&self, data: Vec<T>) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.ubo);
            gl::BufferSubData(
                gl::UNIFORM_BUFFER,
                0,
                (self.size * size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as *const GLvoid,
            );
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }
}

impl<T> Drop for UnifBufArray<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.ubo);
        }
    }
}
