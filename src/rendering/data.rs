use crate::ecs::component::utils::{Color32, Filtering, Texture, Wrapping};
use crate::ecs::component::*;
use crate::glm;
use crate::rendering::sprite_renderer::SpriteSheet;
use crate::utils::constants::*;
use crate::utils::file::*;
use gl::types::*;
use stb_image::image::Image;
use std::collections::HashMap;
use std::path::Path;
use std::ptr;
use std::rc::Rc;

/// generates a gl texture from given image data, filtering and wrapping
#[rustfmt::skip]
fn generate_texture(data: &Image<u8>, filtering: &Filtering, wrapping: &Wrapping) -> GLuint {
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
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint);
            }
            Filtering::Nearest => {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_LINEAR as GLint);
            }
        }
        match wrapping {
            Wrapping::Repeat => {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            }
            Wrapping::MirroredRepeat => {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::MIRRORED_REPEAT as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::MIRRORED_REPEAT as GLint);
            }
            Wrapping::ClampToEdge => {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
            }
            Wrapping::ClampToBorder => {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as GLint);
            }
        }
    }
    tex_id
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
    transparency_map: HashMap<Texture, bool>,
}

impl TextureMap {
    /// creates a new texture map
    pub(crate) fn new() -> Self {
        Self {
            textures: HashMap::new(),
            transparency_map: HashMap::new(),
        }
    }

    /// adds a texture from file
    pub(crate) fn add_texture(&mut self, texture: &Texture) {
        log::debug!("loaded texture: '{:?}'", texture);
        let image = stbi_load_u8_rgba(&texture.path).expect("error loading texture");
        let transparent = image.data.iter().skip(3).step_by(4).any(|a| *a < 255);
        let tex_id = generate_texture(&image, &texture.filtering, &texture.wrapping);
        self.textures.insert(texture.clone(), tex_id);
        self.transparency_map.insert(texture.clone(), transparent);
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
                unsafe { gl::DeleteTextures(1, id) };
                self.transparency_map.remove(texture).unwrap();
            }
            contains
        });
    }

    /// yields a texture id for given name
    pub(crate) fn get_tex_id(&self, texture: &Texture) -> Option<GLuint> {
        self.textures.get(texture).copied()
    }

    /// returns wether or not the texture contains transparency (a < 255)
    pub(crate) fn is_transparent(&self, texture: &Texture) -> bool {
        *self.transparency_map.get(texture).unwrap()
    }

    /// clears the texture map and deletes all of the stored textures
    pub(crate) fn clear(&mut self) {
        for texture in self.textures.values() {
            unsafe { gl::DeleteTextures(1, texture) };
        }
        self.textures.clear();
        self.transparency_map.clear();
    }
}

impl Drop for TextureMap {
    fn drop(&mut self) {
        for (_, texture) in self.textures.iter() {
            unsafe { gl::DeleteTextures(1, texture) };
        }
    }
}

/// stores the sprite sheet data for sprite rendering
pub(crate) struct SpriteTextureMap {
    sheets: HashMap<Rc<Path>, SpriteSheet>,
    sprites: HashMap<Rc<Path>, GLuint>,
}

impl SpriteTextureMap {
    /// creates a new sprite texture map
    pub(crate) fn new() -> Self {
        Self {
            sheets: HashMap::new(),
            sprites: HashMap::new(),
        }
    }

    /// adds a new sprite sheet to the map
    pub(crate) fn add_sheet(&mut self, path: Rc<Path>) {
        log::debug!("loaded sprite sheet: {:?}", path.to_str().unwrap());
        let image = stbi_load_u8_rgba(&path).expect("error loading texture");
        let tex_id = generate_texture(&image, &Filtering::Nearest, &Wrapping::Repeat);
        let sprite_sheet = SpriteSheet {
            texture_id: tex_id,
            data: image,
        };
        self.sheets.insert(path, sprite_sheet);
    }

    /// adds a new sprite texture to the map
    pub(crate) fn add_sprite(&mut self, path: Rc<Path>) {
        log::debug!("loaded sprite: {:?}", path.to_str().unwrap());
        let image = stbi_load_u8_rgba(&path).expect("error loading texture");
        let tex_id = generate_texture(&image, &Filtering::Nearest, &Wrapping::Repeat);
        self.sprites.insert(path, tex_id);
    }

    /// deletes a stored sheets based on a function bool return
    pub(crate) fn retain_sheets<F>(&mut self, mut f: F)
    where
        F: FnMut(&Rc<Path>) -> bool,
    {
        self.sheets.retain(|path, sheet| {
            let contains = f(path);
            if !contains {
                log::debug!("deleted sprite sheet: {:?}", path);
                unsafe { gl::DeleteTextures(1, &sheet.texture_id) };
            }
            contains
        });
    }

    /// deletes a stored sprite textures based on a function bool return
    pub(crate) fn retain_sprites<F>(&mut self, mut f: F)
    where
        F: FnMut(&Rc<Path>) -> bool,
    {
        self.sprites.retain(|path, id| {
            let contains = f(path);
            if !contains {
                log::debug!("deleted sprite: {:?}", path);
                unsafe { gl::DeleteTextures(1, id) };
            }
            contains
        });
    }

    /// yields a sheet reference for given path
    pub(crate) fn get_sheet(&self, path: &Rc<Path>) -> Option<&SpriteSheet> {
        self.sheets.get(path)
    }

    /// yields the texture id for a sprite
    pub(crate) fn get_sprite_id(&self, path: &Rc<Path>) -> Option<GLuint> {
        self.sprites.get(path).copied()
    }

    /// clears the sprite sheet map and deletes all of the stored textures
    pub(crate) fn clear(&mut self) {
        for tex_id in self.sheets.values().map(|sheet| sheet.texture_id) {
            unsafe { gl::DeleteTextures(1, &tex_id) };
        }
        for tex_id in self.sprites.values() {
            unsafe { gl::DeleteTextures(1, tex_id) };
        }
        self.sheets.clear();
        self.sprites.clear();
    }
}

impl Drop for SpriteTextureMap {
    fn drop(&mut self) {
        for tex_id in self.sheets.values().map(|sheet| sheet.texture_id) {
            unsafe { gl::DeleteTextures(1, &tex_id) };
        }
        for tex_id in self.sprites.values() {
            unsafe { gl::DeleteTextures(1, tex_id) };
        }
    }
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
    pub(crate) projection: glm::Mat4,
    pub(crate) view: glm::Mat4,
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

/// stores the current camera config for 2D rendering
pub(crate) struct OrthoCamera {
    pub(crate) projection: glm::Mat4,
    pub(crate) view: glm::Mat4,
}

impl OrthoCamera {
    /// creates a new orthographic camera
    pub(crate) fn new(left: f32, right: f32, bottom: f32, top: f32) -> Self {
        let position = -Z_AXIS;
        Self {
            projection: glm::ortho(left, right, bottom, top, -1.0, 1.0),
            view: glm::look_at(&position, &ORIGIN, &Y_AXIS),
        }
    }

    /// creates a new orthographic camera from a size: `(-size, size, -size, size)`
    pub(crate) fn from_size(size: f32) -> Self {
        Self::new(-size, size, -size, size)
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
    #[rustfmt::skip]
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
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as GLint);
            let border_color = Color32::WHITE.to_vec4();
            gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, border_color.as_ptr());

            gl::BindFramebuffer(gl::FRAMEBUFFER, dbo);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, shadow_map, 0);
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
        unsafe { gl::UniformMatrix4fv(33, 1, gl::FALSE, &self.light_matrix[0]) };
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
        unsafe { gl::DeleteBuffers(1, &self.ubo) };
    }
}

/// a sky box that can be added to the rendering system
pub struct Skybox {
    cube_map: GLuint,
    vao: GLuint,
    vbo: GLuint,
}

impl Skybox {
    /// creates a new skybox cube map from input texture paths ``[right, left, top, bottom, front, back]``
    #[rustfmt::skip]
    pub fn new(paths: [impl AsRef<Path>; 6]) -> Self {
        let mut cube_map = 0;
        unsafe {
            gl::GenTextures(1, &mut cube_map);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, cube_map);
        }
        for i in 0..6 {
            let texture =
                stbi_load_u8_rgba(&paths[i as usize]).expect("error loading skybox texture");
            unsafe {
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
                    0,
                    gl::RGBA8 as GLint,
                    texture.width as GLint,
                    texture.height as GLint,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    texture.data.as_ptr() as *const GLvoid,
                );
            }
        }
        unsafe {
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as GLint);
        }
        let mut vao = 0;
        let mut vbo = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::CreateBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (108 * size_of::<f32>()) as GLsizeiptr,
                SKYBOX_VERTICES.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE as GLboolean, 0, ptr::null());
            gl::BindVertexArray(0);
        }
        log::debug!("created skybox");

        Self { cube_map, vao, vbo }
    }

    /// renders the skybox
    pub(crate) fn render(&self) {
        unsafe {
            gl::DepthFunc(gl::LEQUAL);
            gl::DepthMask(gl::FALSE);
            gl::BindVertexArray(self.vao);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.cube_map);
            gl::Uniform1i(0, 0);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::BindVertexArray(0);
            gl::DepthMask(gl::TRUE);
            gl::DepthFunc(gl::LESS);
        }
    }
}

impl Drop for Skybox {
    fn drop(&mut self) {
        log::debug!("dropped skybox");
        unsafe {
            gl::DeleteTextures(1, &self.cube_map);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

/// the screen texture used for 3D rendering
pub(crate) struct ScreenTexture {
    multi_fbo: GLuint,
    multi_texture: GLuint,
    fbo: GLuint,
    texture: GLuint,
    rbo: GLuint,
    vao: GLuint,
    vbo: GLuint,
    pub(crate) width: GLsizei,
    pub(crate) height: GLsizei,
    tmp_viewport: [GLint; 4],
    msaa: bool,
}

impl ScreenTexture {
    /// creates a new screen texture with a width and height
    #[rustfmt::skip]
    pub(crate) fn new(width: GLsizei, height: GLsizei, msaa: bool) -> Self {
        let mut multi_fbo = 0;
        let mut multi_texture = 0;
        let mut fbo = 0;
        let mut texture = 0;
        let mut rbo = 0;
        let mut vao = 0;
        let mut vbo = 0;
        unsafe {
            // RENDER BUFFER FOR DEPTH + STENCIL
            gl::GenRenderbuffers(1, &mut rbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
            gl::RenderbufferStorageMultisample(gl::RENDERBUFFER, 4, gl::DEPTH24_STENCIL8, width, height);

            // MULTISAMPLED FRAME BUFFER
            gl::GenFramebuffers(1, &mut multi_fbo);
            gl::GenTextures(1, &mut multi_texture);
            gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, multi_texture);
            gl::TexImage2DMultisample(
                gl::TEXTURE_2D_MULTISAMPLE,
                4,
                gl::RGBA,
                width,
                height,
                gl::TRUE
            );
            gl::BindFramebuffer(gl::FRAMEBUFFER, multi_fbo);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D_MULTISAMPLE, multi_texture, 0);
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo);
            assert_eq!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER), gl::FRAMEBUFFER_COMPLETE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // FRAME BUFFER
            gl::GenFramebuffers(1, &mut fbo);
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null()
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture, 0);
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo);
            assert_eq!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER), gl::FRAMEBUFFER_COMPLETE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // VERTEX BUFFER
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::CreateBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (15 * size_of::<f32>()) as GLsizeiptr,
                SCREEN_TRIANGLE_VERTICES.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                (5 * size_of::<f32>()) as GLsizei,
                ptr::null()
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                (5 * size_of::<f32>()) as GLsizei,
                (3 * size_of::<f32>()) as *const GLvoid
            );
            gl::BindVertexArray(0);
        }

        Self {
            multi_fbo,
            multi_texture,
            fbo,
            texture,
            rbo,
            vao,
            vbo,
            width,
            height,
            tmp_viewport: [0; 4],
            msaa
        }
    }

    /// bind the screen texture for rendering
    pub(crate) fn bind(&mut self) {
        unsafe {
            gl::GetIntegerv(gl::VIEWPORT, &mut self.tmp_viewport[0]);
            gl::Viewport(0, 0, self.width, self.height);
            gl::Scissor(0, 0, self.width, self.height);
            if self.msaa {
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.multi_fbo);
            } else {
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            }
        }
    }

    /// unbind the screen texture and use the default frame buffer
    #[rustfmt::skip]
    pub(crate) fn unbind(&self) {
        unsafe {
            if self.msaa {
                gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.multi_fbo);
                gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.fbo);
                gl::BlitFramebuffer(0, 0, self.width, self.height, 0, 0, self.width, self.height, gl::COLOR_BUFFER_BIT, gl::NEAREST);
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
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

    /// render the screen texture triangle
    pub(crate) fn render(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindTextureUnit(0, self.texture);
            gl::Uniform1i(0, 0);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for ScreenTexture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteTextures(1, &self.multi_texture);
            gl::DeleteRenderbuffers(1, &self.rbo);
            gl::DeleteFramebuffers(1, &self.multi_fbo);
            gl::DeleteTextures(1, &self.texture);
            gl::DeleteFramebuffers(1, &self.fbo);
        }
    }
}
