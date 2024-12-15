use crate::ecs::component::utils::{Color32, Filtering};
use crate::ecs::component::*;
use crate::glm;
use crate::utils::constants::*;
use crate::utils::file::*;
use gl::types::*;
use stb_image::image::{Image, LoadResult};
use std::collections::HashMap;
use std::path::Path;
use std::ptr;

/// generates a gl texture from given image data
fn generate_texture(data: Image<u8>, filtering: &Filtering) -> GLuint {
    let mut tex_id = 0;
    unsafe {
        gl::GenTextures(1, &mut tex_id);
        gl::BindTexture(gl::TEXTURE_2D, tex_id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA8 as GLint,
            data.width as GLint,
            data.height as GLint,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data.data.as_ptr() as *const GLvoid,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
        match filtering {
            Filtering::Linear => {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_MIN_FILTER,
                    gl::LINEAR_MIPMAP_LINEAR as GLint,
                );
            }
            Filtering::Nearest => {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_MIN_FILTER,
                    gl::NEAREST_MIPMAP_LINEAR as GLint,
                );
            }
        }
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
    }
    tex_id
}

/// loads an opengl texture from a file path
pub(crate) fn load_texture_from_path(file_path: impl AsRef<Path>, filtering: &Filtering) -> GLuint {
    let texture: Image<u8>;
    match stb_image::image::load_with_depth(file_path, 4, false) {
        LoadResult::ImageU8(im) => {
            texture = im;
        }
        _ => {
            panic!("error loading texture")
        }
    }
    generate_texture(texture, filtering)
}

/// loads an opengl texture from bytes
pub(crate) fn load_texture_from_bytes(bytes: &[u8], filtering: &Filtering) -> GLuint {
    let texture: Image<u8>;
    match stb_image::image::load_from_memory_with_depth(bytes, 4, false) {
        LoadResult::ImageU8(im) => {
            texture = im;
        }
        _ => {
            panic!("error loading texture from memory")
        }
    }
    generate_texture(texture, filtering)
}

/// data for a single vertex
#[derive(Default, Clone, Copy, Debug)]
#[repr(C)]
pub(crate) struct Vertex {
    pub(crate) position: glm::Vec3,
    pub(crate) color: glm::Vec4,
    pub(crate) uv_coords: glm::Vec2,
    pub(crate) normal: glm::Vec3,
    pub(crate) tex_index: GLfloat,
}

/// holds the texture ID's for the App
pub(crate) struct TextureMap {
    textures: HashMap<Texture, GLuint>,
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
        log::debug!("loaded texture: '{:?}'", texture);
        match texture {
            Texture::Ice(filtering) => {
                self.textures.insert(
                    texture.clone(),
                    load_texture_from_bytes(ICE_TEXTURE, filtering),
                );
            }
            Texture::Sand(filtering) => {
                self.textures.insert(
                    texture.clone(),
                    load_texture_from_bytes(SAND_TEXTURE, filtering),
                );
            }
            Texture::Wall(filtering) => {
                self.textures.insert(
                    texture.clone(),
                    load_texture_from_bytes(WALL_TEXTURE, filtering),
                );
            }
            Texture::Custom(path, filtering) => {
                self.textures
                    .insert(texture.clone(), load_texture_from_path(path, filtering));
            }
        }
    }

    /// deletes a stored textures based on a function bool return
    pub(crate) fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Texture) -> bool,
    {
        self.textures.retain(|texture, id| {
            let contains = f(texture);
            if !contains {
                log::debug!("deleted texture: '{:?}'", texture);
                unsafe {
                    gl::DeleteTextures(1, id);
                }
            }
            contains
        });
    }

    /// yields a texture id for given name
    pub(crate) fn get_tex_id(&self, texture: &Texture) -> Option<GLuint> {
        self.textures.get(texture).copied()
    }

    /// clears the texture map and deletes all of the stored textures
    pub(crate) fn clear(&mut self) {
        unsafe {
            for (_, texture) in self.textures.iter() {
                gl::DeleteTextures(1, texture);
            }
        }
        self.textures.clear();
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
    center_of_mass: &glm::Vec3,
) -> glm::Mat4 {
    let mass_offset = glm::translate(&glm::Mat4::identity(), center_of_mass);
    let inv_mass_offset = mass_offset.try_inverse().unwrap();
    let translate = glm::translate(&glm::Mat4::identity(), position.data());
    let rotate = orientation.rotation_matrix();
    let scaled = scale.scale_matrix();
    translate * mass_offset * rotate * inv_mass_offset * scaled
}

/// stores the current camera config for 3D rendering
pub(crate) struct PerspectiveCamera {
    projection: glm::Mat4,
    view: glm::Mat4,
    win_width: f32,
    win_height: f32,
    fov: f32,
}

impl PerspectiveCamera {
    /// creates new config with default values
    pub(crate) fn new(position: glm::Vec3, focus: glm::Vec3) -> Self {
        let win_width = MIN_WIN_WIDTH as f32;
        let win_height = MIN_WIN_HEIGHT as f32;
        let fov = 45.0_f32.to_radians();
        let projection = glm::perspective::<f32>(win_width / win_height, fov, 0.1, 100.0);
        let view = glm::look_at::<f32>(&position, &focus, &Y_AXIS);

        Self {
            projection,
            view,
            win_width,
            win_height,
            fov,
        }
    }

    /// update the projection matrix based on a given fov
    pub(crate) fn update_fov(&mut self, fov: f32) {
        self.fov = fov.to_radians();
        self.recompute_projection();
    }

    /// updates the internally stored values for the window size and recompute the projection
    pub(crate) fn update_win_size(&mut self, win_width: u32, win_height: u32) {
        self.win_width = win_width as f32;
        self.win_height = win_height as f32;
        self.recompute_projection();
    }

    /// updates the camera for given camera position and focus
    pub(crate) fn update_cam(&mut self, position: &glm::Vec3, focus: &glm::Vec3) {
        self.view = glm::look_at(position, focus, &Y_AXIS);
    }

    /// refreshes the stored projection matrix
    fn recompute_projection(&mut self) {
        self.projection =
            glm::perspective::<f32>(self.win_width / self.win_height, self.fov, 0.1, 100.0);
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
    pub(crate) light_pos: glm::Vec3,
    pub(crate) light: PointLight,
    tmp_viewport: [GLint; 4],
}

impl ShadowMap {
    /// creates a new shadow map with given size (width, height)
    pub(crate) fn new(size: (GLsizei, GLsizei), light_pos: glm::Vec3, light: &PointLight) -> Self {
        log::debug!("created new shadow map");
        let mut dbo = 0;
        let mut shadow_map = 0;

        unsafe {
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

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
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
        let dot = light.direction.normalize().dot(&Y_AXIS);
        let from_view_up = if dot.abs() == 1.0 {
            Z_AXIS
        } else {
            (Y_AXIS - dot * Y_AXIS).normalize()
        };

        Self {
            dbo,
            shadow_map,
            size,
            light_matrix: glm::perspective::<f32>(1.0, 90f32.to_radians(), 0.1, 100.0)
                * glm::look_at(&light_pos, &(light_pos + light.direction), &from_view_up),
            light_pos,
            light: *light,
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
            // clear the depth buffer bit
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }
    }

    /// binds the light matrix uniform to the currently used shader
    pub(crate) fn bind_light_matrix(&self) {
        unsafe {
            gl::UniformMatrix4fv(33, 1, gl::FALSE, &self.light_matrix[0]);
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

    /// updates the shadow map according to a new light data
    pub(crate) fn update_light(&mut self, pos: &glm::Vec3, light: &PointLight) {
        self.light_pos = *pos;
        self.light = *light;

        let dot = light.direction.normalize().dot(&Y_AXIS);
        let from_view_up = if dot.abs() == 1.0 {
            Z_AXIS
        } else {
            (Y_AXIS - dot * Y_AXIS).normalize()
        };

        self.light_matrix = glm::perspective::<f32>(1.0, 90f32.to_radians(), 0.1, 100.0)
            * glm::look_at(pos, &(pos + light.direction), &from_view_up);
    }
}

impl Drop for ShadowMap {
    fn drop(&mut self) {
        log::debug!("dropped shadow map");
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
    pub(crate) color: glm::Vec4,
    pub(crate) intensity: GLfloat,
    pub(crate) padding_12bytes: glm::Vec3, // necessary for std140 uniform buffer layout padding
}

/// light source data for uniform buffer use
#[repr(C)]
pub(crate) struct LightConfig {
    pub(crate) color: glm::Vec4,
    pub(crate) intensity: GLfloat,
}

/// uniform buffer wrapper for one array of uniforms
pub(crate) struct UniformBuffer {
    pub(crate) ubo: GLuint,
}

impl UniformBuffer {
    /// creates a new uniform buffer with 'size' bytes
    pub(crate) fn new(size: usize) -> Self {
        let mut ubo = 0;
        unsafe {
            gl::GenBuffers(1, &mut ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, ubo);
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                size as GLsizeiptr,
                ptr::null(),
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
        Self { ubo }
    }

    /// uploads data to the buffer
    pub(crate) fn upload_data(&self, offset: usize, size: usize, ptr: *const GLvoid) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.ubo);
            gl::BufferSubData(
                gl::UNIFORM_BUFFER,
                offset as GLsizeiptr,
                size as GLsizeiptr,
                ptr,
            );
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }
}

impl Drop for UniformBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.ubo);
        }
    }
}
