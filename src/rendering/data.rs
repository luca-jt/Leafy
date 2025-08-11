use crate::internal_prelude::*;
use crate::rendering::shader::ShaderProgram;
use crate::rendering::sprite_renderer::SpriteSheet;
use stb_image::image::Image;
use std::ptr;

/// generates a gl texture from given image data, filtering and wrapping
#[rustfmt::skip]
fn generate_texture(data: &Image<u8>, filtering: Filtering, wrapping: Wrapping, color_space: ColorSpace) -> GLuint {
    let gl_color_space_enum = match color_space {
        ColorSpace::SRGBA => gl::SRGB_ALPHA as GLint,
        ColorSpace::RGBA8 => gl::RGBA8 as GLint,
    };
    let mut tex_id = 0;
    unsafe {
        gl::GenTextures(1, &mut tex_id);
        gl::BindTexture(gl::TEXTURE_2D, tex_id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl_color_space_enum,
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

/// holds the texture ID's for the App
pub(crate) struct TextureMap {
    textures: AHashMap<Texture, GLuint>,
    material_textures: AHashMap<String, GLuint>,
    sheets: AHashMap<Rc<Path>, SpriteSheet>,
    sprites: AHashMap<Rc<Path>, GLuint>,
}

impl TextureMap {
    /// creates a new texture map
    pub(crate) fn new() -> Self {
        Self {
            textures: AHashMap::new(),
            material_textures: AHashMap::new(),
            sheets: AHashMap::new(),
            sprites: AHashMap::new(),
        }
    }

    /// adds a texture from file
    pub(crate) fn add_texture(&mut self, texture: &Texture) -> bool {
        if self.textures.contains_key(texture) {
            log::warn!("Texture data for {texture:?} already present.");
            return false;
        }
        if let Some(image) = stbi_load_u8_rgba(&texture.path) {
            let tex_id = generate_texture(
                &image,
                texture.filtering,
                texture.wrapping,
                texture.color_space,
            );
            self.textures.insert(texture.clone(), tex_id);
            log::debug!("Loaded texture: {texture:?}");
            true
        } else {
            log::error!("Error loading texture file data for {texture:?}");
            false
        }
    }

    /// deletes a stored texture
    pub(crate) fn delete_texture(&mut self, texture: &Texture) -> bool {
        if let Some(id) = self.textures.remove(texture) {
            unsafe { gl::DeleteTextures(1, &id) };
            log::debug!("Deleted texture: {texture:?}");
            true
        } else {
            log::warn!("Texture data not present for {texture:?}");
            false
        }
    }

    /// loads a texture that is part of materials and is used in rendering
    pub(crate) fn add_material_texture(&mut self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();
        if let Some(image) = stbi_load_u8_rgba(path) {
            let tex_id = generate_texture(
                &image,
                Filtering::Nearest,
                Wrapping::default(),
                ColorSpace::RGBA8,
            );
            log::debug!("Loaded material texture: {path:?}.");
            self.material_textures
                .insert(path.file_name().unwrap().to_str().unwrap().into(), tex_id);
            true
        } else {
            log::error!("Error loading material texture file data from {path:?}.");
            false
        }
    }

    /// delete a stored material texture
    pub(crate) fn delete_material_texture(&mut self, name: impl AsRef<str>) -> bool {
        let name = name.as_ref();
        if let Some(id) = self.material_textures.remove(name) {
            unsafe { gl::DeleteTextures(1, &id) };
            log::debug!("Deleted material texture: {name:?}.");
            true
        } else {
            log::warn!("Material texture data not present for name: {name:?}.");
            false
        }
    }

    /// yields a texture id for given name
    pub(crate) fn get_tex_id(&self, texture: &Texture) -> Option<GLuint> {
        let result = self.textures.get(texture).copied();
        if result.is_none() {
            log::warn!("Texture {texture:?} is not loaded!");
        }
        result
    }

    /// yields a material texture id for given name
    pub(crate) fn get_material_tex_id(&self, name: impl AsRef<str>) -> Option<GLuint> {
        let name = name.as_ref();
        let result = self.material_textures.get(name).copied();
        if result.is_none() {
            log::warn!("Material texture {name:?} is not loaded!");
        }
        result
    }

    /// adds a new sprite sheet to the map
    pub(crate) fn add_sheet(&mut self, path: &Rc<Path>) -> bool {
        let path_str = path.to_str().unwrap();
        if self.sheets.contains_key(path) {
            log::warn!("Sprite sheet data already present for source: {path_str:?}.");
            return false;
        }
        if let Some(image) = stbi_load_u8_rgba(path) {
            let tex_id = generate_texture(
                &image,
                Filtering::Nearest,
                Wrapping::Repeat,
                ColorSpace::RGBA8,
            );
            let sprite_sheet = SpriteSheet {
                texture_id: tex_id,
                width: image.width,
                height: image.height,
            };
            log::debug!("Loaded sprite sheet: {path_str:?}.");
            self.sheets.insert(path.clone(), sprite_sheet);
            true
        } else {
            log::error!("Error loading sprite sheet file data from: {path_str:?}.");
            false
        }
    }

    /// deletes a stored sheets based on a function bool return
    pub(crate) fn delete_sheet(&mut self, path: &Rc<Path>) -> bool {
        let path_str = path.to_str().unwrap();
        if let Some(sheet) = self.sheets.remove(path) {
            unsafe { gl::DeleteTextures(1, &sheet.texture_id) };
            log::debug!("Deleted sprite sheet: {path_str:?}.");
            true
        } else {
            log::warn!("Sprite sheet data not present for source: {path_str:?}.");
            false
        }
    }

    /// adds a new sprite texture to the map
    pub(crate) fn add_sprite(&mut self, path: &Rc<Path>) -> bool {
        let path_str = path.to_str().unwrap();
        if self.sprites.contains_key(path) {
            log::warn!("Sprite sheet data already present for source: {path_str:?}.");
            return false;
        }
        if let Some(image) = stbi_load_u8_rgba(path) {
            let tex_id = generate_texture(
                &image,
                Filtering::Nearest,
                Wrapping::Repeat,
                ColorSpace::RGBA8,
            );
            log::debug!("Loaded sprite: {path_str:?}.");
            self.sprites.insert(path.clone(), tex_id);
            true
        } else {
            log::error!("Error loading sprite file data from source: {path_str:?}.");
            false
        }
    }

    /// deletes a stored sprite textures based on a function bool return
    pub(crate) fn delete_sprite(&mut self, path: &Rc<Path>) -> bool {
        let path_str = path.to_str().unwrap();
        if let Some(id) = self.sprites.remove(path) {
            unsafe { gl::DeleteTextures(1, &id) };
            log::debug!("Deleted sprite: {path_str:?}.");
            true
        } else {
            log::warn!("Sprite data not present for source: {path_str:?}.");
            false
        }
    }

    /// yields a sheet reference for given path
    pub(crate) fn get_sheet(&self, path: &Rc<Path>) -> Option<&SpriteSheet> {
        let result = self.sheets.get(path);
        if result.is_none() {
            log::warn!("Sprite sheet {path:?} is not loaded!");
        }
        result
    }

    /// yields the texture id for a sprite
    pub(crate) fn get_sprite_id(&self, path: &Rc<Path>) -> Option<GLuint> {
        let result = self.sprites.get(path).copied();
        if result.is_none() {
            log::warn!("Sprite {path:?} is not loaded!");
        }
        result
    }

    /// clears the texture map and deletes all of the stored textures
    pub(crate) fn clear(&mut self) {
        for texture in self.textures.values() {
            unsafe { gl::DeleteTextures(1, texture) };
        }
        for texture in self.material_textures.values() {
            unsafe { gl::DeleteTextures(1, texture) };
        }
        self.textures.clear();
        self.material_textures.clear();

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

impl Drop for TextureMap {
    fn drop(&mut self) {
        for tex_id in self.textures.values() {
            unsafe { gl::DeleteTextures(1, tex_id) };
        }
        for tex_id in self.material_textures.values() {
            unsafe { gl::DeleteTextures(1, tex_id) };
        }

        for tex_id in self.sheets.values().map(|sheet| sheet.texture_id) {
            unsafe { gl::DeleteTextures(1, &tex_id) };
        }
        for tex_id in self.sprites.values() {
            unsafe { gl::DeleteTextures(1, tex_id) };
        }
    }
}

/// Calculate the model matrix for a given ``Position``, ``Scale``, ``Orientation``, and center of mass.
pub fn calc_model_matrix(
    position: &Position,
    scale: &Scale,
    orientation: &Orientation,
    center_of_mass: &Vec3,
) -> Mat4 {
    let mass_offset = glm::translate(&Mat4::identity(), center_of_mass);
    let inv_mass_offset = mass_offset.try_inverse().unwrap();
    let translate = glm::translate(&Mat4::identity(), position.data());
    let rotate = orientation.rotation_matrix();
    let scaled = scale.scale_matrix();
    translate * mass_offset * rotate * inv_mass_offset * scaled
}

/// stores the current camera config for 3D rendering
pub(crate) struct PerspectiveCamera {
    pub(crate) projection: Mat4,
    pub(crate) view: Mat4,
    viewport_ratio: f32,
    fov: f32,
}

impl PerspectiveCamera {
    /// creates new config with default values
    pub(crate) fn new() -> Self {
        let viewport_ratio = DEFAULT_WIN_WIDTH as f32 / DEFAULT_WIN_HEIGHT as f32;
        let fov = 45.0_f32.to_radians();
        let projection = glm::perspective(viewport_ratio, fov, NEAR_PLANE, FAR_PLANE);
        let view = glm::look_at::<f32>(&-Z_AXIS, &ORIGIN, &Y_AXIS);

        Self {
            projection,
            view,
            viewport_ratio,
            fov,
        }
    }

    /// update the projection matrix based on a given fov
    pub(crate) fn update_fov(&mut self, fov: f32) {
        self.fov = fov.to_radians();
        self.projection = glm::perspective(self.viewport_ratio, self.fov, NEAR_PLANE, FAR_PLANE);
    }

    /// updates the internally stored values for the window size and recompute the projection
    pub(crate) fn update_win_size(&mut self, viewport_ratio: f32) {
        self.viewport_ratio = viewport_ratio;
        self.projection = glm::perspective(self.viewport_ratio, self.fov, NEAR_PLANE, FAR_PLANE);
    }

    /// updates the camera for given camera position and focus
    pub(crate) fn update_cam(&mut self, position: &Vec3, focus: &Vec3, up: &Vec3) {
        self.view = glm::look_at(position, focus, up);
    }

    /// current FOV in degrees
    #[inline]
    pub(crate) fn fov(&self) -> f32 {
        self.fov.to_degrees()
    }
}

/// stores the current camera config for 2D rendering
pub(crate) struct OrthoCamera {
    pub(crate) projection: Mat4,
    pub(crate) view: Mat4,
}

impl OrthoCamera {
    /// creates a new orthographic camera
    pub(crate) fn new(left: f32, right: f32) -> Self {
        let position = Z_AXIS;
        Self {
            projection: glm::ortho(left, right, -1.0, 1.0, NEAR_PLANE, FAR_PLANE_SPRITE),
            view: glm::look_at(&position, &ORIGIN, &Y_AXIS),
        }
    }

    /// updates the internally stored values for the window size and recompute the projection
    pub(crate) fn update_win_size(&mut self, viewport_ratio: f32) {
        self.projection = glm::ortho(
            -viewport_ratio,
            viewport_ratio,
            -1.0,
            1.0,
            NEAR_PLANE,
            FAR_PLANE_SPRITE,
        );
    }
}

/// shadow cube map for point lights that is shadow map agnostic
#[repr(C)]
pub(crate) struct CubeShadowMap {
    dbo: GLuint,
    shadow_cube_map: GLuint,
    side_size: (GLsizei, GLsizei),
    pub(crate) base_light_matrices: [Mat4; 6],
    light_pos: Vec3,
    tmp_viewport: [GLint; 4],
}

impl CubeShadowMap {
    /// creates a new shadow cube map with given side size (width, height)
    #[rustfmt::skip]
    pub(crate) fn new(side_size: (GLsizei, GLsizei), light_pos: Vec3) -> Self {
        log::debug!("Created new shadow cube map.");
        let mut dbo = 0;
        let mut shadow_cube_map = 0;

        unsafe {
            gl::GenFramebuffers(1, &mut dbo);
            gl::GenTextures(1, &mut shadow_cube_map);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, shadow_cube_map);
            for i in 0..6 {
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
                    0,
                    gl::DEPTH_COMPONENT as GLint,
                    side_size.0,
                    side_size.1,
                    0,
                    gl::DEPTH_COMPONENT,
                    gl::FLOAT,
                    ptr::null(),
                );
            }
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as GLint);

            gl::BindFramebuffer(gl::FRAMEBUFFER, dbo);
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, shadow_cube_map, 0);
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        let projection = glm::perspective(1.0, 90f32.to_radians(), NEAR_PLANE, FAR_PLANE);

        Self {
            dbo,
            shadow_cube_map,
            side_size,
            base_light_matrices: [
                projection * glm::look_at(&light_pos, &(light_pos + X_AXIS), &-Y_AXIS),
                projection * glm::look_at(&light_pos, &(light_pos - X_AXIS), &-Y_AXIS),
                projection * glm::look_at(&light_pos, &(light_pos + Y_AXIS), &Z_AXIS),
                projection * glm::look_at(&light_pos, &(light_pos - Y_AXIS), &-Z_AXIS),
                projection * glm::look_at(&light_pos, &(light_pos + Z_AXIS), &-Y_AXIS),
                projection * glm::look_at(&light_pos, &(light_pos - Z_AXIS), &-Y_AXIS),
            ],
            light_pos,
            tmp_viewport: [0; 4],
        }
    }

    /// bind the depth buffer for writing
    pub(crate) fn bind_writing(&mut self) {
        unsafe {
            gl::GetIntegerv(gl::VIEWPORT, &mut self.tmp_viewport[0]);
            gl::Viewport(0, 0, self.side_size.0, self.side_size.1);
            gl::Scissor(0, 0, self.side_size.0, self.side_size.1);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.dbo);
            // clear the depth buffer bit
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }
    }

    /// binds the light uniforms to the currently used shader
    pub(crate) fn bind_light_uniforms(&self) {
        unsafe {
            gl::UniformMatrix4fv(0, 6, gl::FALSE, &self.base_light_matrices[0][0]);
            gl::Uniform3fv(46, 1, &self.light_pos[0]);
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
        gl::ActiveTexture(gl::TEXTURE0 + texture_unit);
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.shadow_cube_map);
    }

    /// updates the shadow map according to a new light data
    pub(crate) fn update_light(&mut self, pos: &Vec3) {
        self.light_pos = *pos;
        let projection = glm::perspective(1.0, 90f32.to_radians(), NEAR_PLANE, FAR_PLANE);
        self.base_light_matrices = [
            projection * glm::look_at(&self.light_pos, &(self.light_pos + X_AXIS), &-Y_AXIS),
            projection * glm::look_at(&self.light_pos, &(self.light_pos - X_AXIS), &-Y_AXIS),
            projection * glm::look_at(&self.light_pos, &(self.light_pos + Y_AXIS), &Z_AXIS),
            projection * glm::look_at(&self.light_pos, &(self.light_pos - Y_AXIS), &-Z_AXIS),
            projection * glm::look_at(&self.light_pos, &(self.light_pos + Z_AXIS), &-Y_AXIS),
            projection * glm::look_at(&self.light_pos, &(self.light_pos - Z_AXIS), &-Y_AXIS),
        ];
    }
}

impl Drop for CubeShadowMap {
    fn drop(&mut self) {
        log::debug!("Dropped shadow cube map.");
        unsafe {
            gl::DeleteTextures(1, &self.shadow_cube_map);
            gl::DeleteFramebuffers(1, &self.dbo);
        }
    }
}

/// shadow map used for directional lights in rendering
pub(crate) struct ShadowMap {
    dbo: GLuint,
    shadow_map: GLuint,
    size: (GLsizei, GLsizei),
    pub(crate) light_matrix: Mat4,
    pub(crate) light_pos: Vec3,
    pub(crate) light: DirectionalLight,
    tmp_viewport: [GLint; 4],
}

impl ShadowMap {
    /// creates a new shadow map with given size (width, height)
    #[rustfmt::skip]
    pub(crate) fn new(size: (GLsizei, GLsizei), light_pos: Vec3, light: &DirectionalLight) -> Self {
        log::debug!("Created new shadow map for a directional light.");
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
            light_matrix: glm::ortho(-10.0, 10.0, -10.0, 10.0, NEAR_PLANE, FAR_PLANE)
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
        unsafe { gl::UniformMatrix4fv(0, 1, gl::FALSE, &self.light_matrix[0]) };
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
        gl::ActiveTexture(gl::TEXTURE0 + texture_unit);
        gl::BindTexture(gl::TEXTURE_2D, self.shadow_map);
    }

    /// updates the shadow map according to a new light data
    pub(crate) fn update_light(&mut self, pos: &Vec3, light: &DirectionalLight) {
        self.light_pos = *pos;
        self.light = *light;

        let dot = light.direction.normalize().dot(&Y_AXIS);
        let from_view_up = if dot.abs() == 1.0 {
            Z_AXIS
        } else {
            (Y_AXIS - dot * Y_AXIS).normalize()
        };

        self.light_matrix = glm::ortho(-10.0, 10.0, -10.0, 10.0, NEAR_PLANE, FAR_PLANE)
            * glm::look_at(pos, &(pos + light.direction), &from_view_up);
    }
}

impl Drop for ShadowMap {
    fn drop(&mut self) {
        log::debug!("Dropped directional shadow map.");
        unsafe {
            gl::DeleteTextures(1, &self.shadow_map);
            gl::DeleteFramebuffers(1, &self.dbo);
        }
    }
}

/// one directional light data block for uniform buffer use
#[repr(C)]
pub(crate) struct DirLightData {
    pub(crate) light_pos: Vec4, // position of the light
    pub(crate) light_matrix: Mat4,
    pub(crate) color: Vec4,
    pub(crate) intensity: GLfloat,
    pub(crate) padding_12bytes: Vec3, // necessary for std140 uniform buffer layout padding
    pub(crate) direction: Vec3,
    pub(crate) padding_4bytes: f32, // necessary for std140 uniform buffer layout padding
}

/// one point light data block for uniform buffer use
#[repr(C)]
pub(crate) struct PointLightData {
    pub(crate) light_pos: Vec4, // position of the light
    pub(crate) color: Vec4,
    pub(crate) intensity: GLfloat,
    pub(crate) has_shadows: GLint,
    pub(crate) padding_8bytes: Vec2, // necessary for std140 uniform buffer layout padding
}

/// light source data for uniform buffer use
#[repr(C)]
pub(crate) struct LightConfig {
    pub(crate) color: Vec4,
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

/// A sky box that can be added to the rendering system.
pub struct Skybox {
    cube_map: GLuint,
    vao: GLuint,
    vbo: GLuint,
}

impl Skybox {
    /// Trys to create a new skybox cube map from input texture paths ``[right, left, top, bottom, front, back]``. Returns ``None`` if the texture loading from the given paths was unsuccessful.
    #[rustfmt::skip]
    pub fn try_new(paths: [impl AsRef<Path>; 6]) -> Option<Self> {
        let mut cube_map = 0;
        unsafe {
            gl::GenTextures(1, &mut cube_map);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, cube_map);
        }
        for i in 0..6 {
            let path = paths[i as usize].as_ref();
            if let Some(texture) = stbi_load_u8_rgba(path) {
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
            } else {
                log::warn!("Error loading skybox texture from {path:?}.");
                unsafe { gl::DeleteTextures(1, &cube_map) };
                return None;
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
            gl::GenBuffers(1, &mut vbo);
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
        log::trace!("Created skybox.");

        Some(Self { cube_map, vao, vbo })
    }

    /// renders the skybox
    pub(crate) fn render(&self, bloom_threshold_shift_skybox: GLfloat) {
        unsafe {
            gl::DepthFunc(gl::LEQUAL);
            gl::DepthMask(gl::FALSE);
            gl::BindVertexArray(self.vao);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.cube_map);
            gl::Uniform1i(0, 0);
            gl::Uniform1f(1, bloom_threshold_shift_skybox);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::BindVertexArray(0);
            gl::DepthMask(gl::TRUE);
            gl::DepthFunc(gl::LESS);
        }
    }
}

impl Drop for Skybox {
    fn drop(&mut self) {
        log::trace!("Dropped skybox.");
        unsafe {
            gl::DeleteTextures(1, &self.cube_map);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

/// the screen texture used for 3D rendering
#[repr(C)]
pub(crate) struct ScreenTexture {
    multi_fbo: GLuint,
    multi_texture: GLuint,
    bloom_multi_texture: GLuint,
    fbo: GLuint,
    texture: GLuint,
    bloom_texture: GLuint,
    multi_rbo: GLuint,
    rbo: GLuint,
    vao: GLuint,
    vbo: GLuint,
    width: GLsizei,
    height: GLsizei,
    tmp_viewport: [GLint; 4],
    msaa: bool,
    ping_pong_fbos: [GLuint; 2],
    ping_pong_textures: [GLuint; 2],
    color_attachments: Vec<GLenum>,
}

impl ScreenTexture {
    /// creates a new screen texture with a width and height and msaa config
    #[rustfmt::skip]
    pub(crate) fn new(width: GLsizei, height: GLsizei, msaa: bool, samples: GLsizei) -> Self {
        let mut multi_fbo = 0;
        let mut multi_texture = 0;
        let mut bloom_multi_texture = 0;
        let mut fbo = 0;
        let mut texture = 0;
        let mut bloom_texture = 0;
        let mut multi_rbo = 0;
        let mut rbo = 0;
        let mut vao = 0;
        let mut vbo = 0;
        let mut ping_pong_fbos = [0, 0];
        let mut ping_pong_textures = [0, 0];
        unsafe {
            // BLOOM TEXTURES
            gl::GenTextures(1, &mut bloom_texture);
            gl::BindTexture(gl::TEXTURE_2D, bloom_texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA16F as GLint,
                width,
                height,
                0,
                gl::RGBA,
                gl::FLOAT,
                ptr::null()
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

            gl::GenTextures(1, &mut bloom_multi_texture);
            gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, bloom_multi_texture);
            gl::TexImage2DMultisample(
                gl::TEXTURE_2D_MULTISAMPLE,
                samples,
                gl::RGBA16F,
                width,
                height,
                gl::TRUE
            );

            // RENDER BUFFERS FOR DEPTH + STENCIL MULTISAMPLED
            gl::GenRenderbuffers(1, &mut multi_rbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, multi_rbo);
            gl::RenderbufferStorageMultisample(gl::RENDERBUFFER, 4, gl::DEPTH24_STENCIL8, width, height);

            // MULTISAMPLED FRAME BUFFER WITH TEXTURES
            gl::GenFramebuffers(1, &mut multi_fbo);

            gl::GenTextures(1, &mut multi_texture);
            gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, multi_texture);
            gl::TexImage2DMultisample(
                gl::TEXTURE_2D_MULTISAMPLE,
                samples,
                gl::RGBA16F,
                width,
                height,
                gl::TRUE
            );
            gl::BindFramebuffer(gl::FRAMEBUFFER, multi_fbo);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D_MULTISAMPLE, multi_texture, 0);
            gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, bloom_multi_texture);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D_MULTISAMPLE, bloom_multi_texture, 0);

            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, multi_rbo);
            assert_eq!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER), gl::FRAMEBUFFER_COMPLETE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // RENDER BUFFERS FOR DEPTH + STENCIL
            gl::GenRenderbuffers(1, &mut rbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);

            // FRAME BUFFER WITH TEXTURES
            gl::GenFramebuffers(1, &mut fbo);

            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA16F as GLint,
                width,
                height,
                0,
                gl::RGBA,
                gl::FLOAT,
                ptr::null()
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture, 0);
            gl::BindTexture(gl::TEXTURE_2D, bloom_texture);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, bloom_texture, 0);

            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo);
            assert_eq!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER), gl::FRAMEBUFFER_COMPLETE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // VERTEX BUFFER
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::GenBuffers(1, &mut vbo);
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

            if msaa {
                gl::BindRenderbuffer(gl::RENDERBUFFER, multi_rbo);
            } else {
                gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
            }

            // PING PONG FRAMEBUFFERS
            gl::GenFramebuffers(2, &mut ping_pong_fbos[0]);
            gl::GenTextures(2, &mut ping_pong_textures[0]);
            for i in 0..2 {
                gl::BindFramebuffer(gl::FRAMEBUFFER, ping_pong_fbos[i]);
                gl::BindTexture(gl::TEXTURE_2D, ping_pong_textures[i]);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA16F as GLint,
                    width,
                    height,
                    0,
                    gl::RGBA,
                    gl::FLOAT,
                    ptr::null()
                );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
                gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, ping_pong_textures[i], 0);
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Self {
            multi_fbo,
            multi_texture,
            bloom_multi_texture,
            fbo,
            texture,
            bloom_texture,
            multi_rbo,
            rbo,
            vao,
            vbo,
            width,
            height,
            tmp_viewport: [0; 4],
            msaa,
            ping_pong_fbos,
            ping_pong_textures,
            color_attachments: vec![gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1]
        }
    }

    /// toggles the use of anti-aliasing and switches the render buffers
    pub(crate) fn use_msaa(&mut self, msaa: bool) {
        self.msaa = msaa;
        if msaa {
            unsafe {
                gl::BindRenderbuffer(gl::RENDERBUFFER, self.multi_rbo);
            }
        } else {
            unsafe {
                gl::BindRenderbuffer(gl::RENDERBUFFER, self.rbo);
            }
        }
    }

    /// bind the screen texture for rendering
    pub(crate) fn bind(&mut self, clear_color: Vec4, background_as_scene_element: bool) {
        unsafe {
            gl::GetIntegerv(gl::VIEWPORT, &mut self.tmp_viewport[0]);
            gl::Viewport(0, 0, self.width, self.height);
            gl::Scissor(0, 0, self.width, self.height);
            if self.msaa {
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.multi_fbo);
            } else {
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            }

            gl::DrawBuffer(gl::COLOR_ATTACHMENT0);

            let clear_color_a = if background_as_scene_element {
                clear_color.w
            } else {
                0.0
            };

            gl::ClearColor(clear_color.x, clear_color.y, clear_color.z, clear_color_a);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawBuffer(gl::COLOR_ATTACHMENT1);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::DrawBuffers(2, self.color_attachments.as_ptr());
            gl::ClearColor(clear_color.x, clear_color.y, clear_color.z, clear_color.w);
            gl::Clear(gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }
    }

    /// unbind the screen texture and use the default frame buffer
    #[rustfmt::skip]
    pub(crate) fn unbind(&self, bloom_shader: &ShaderProgram, use_bloom: bool, bloom_iterations: usize) {
        unsafe {
            // blit mulisampled texture if necessary
            if self.msaa {
                gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.multi_fbo);
                gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.fbo);
                for attachment in self.color_attachments.iter().copied() {
                    gl::ReadBuffer(attachment);
                    gl::DrawBuffer(attachment);
                    gl::BlitFramebuffer(0, 0, self.width, self.height, 0, 0, self.width, self.height, gl::COLOR_BUFFER_BIT, gl::NEAREST);
                }
            }

            // bloom blur
            if use_bloom {
                let mut horizontal = true;
                bloom_shader.use_program();
                gl::BindVertexArray(self.vao);
                for i in 0..(bloom_iterations * 2) {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, self.ping_pong_fbos[horizontal as usize]);

                    let source_texture = if i == 0 {
                        self.bloom_texture
                    } else {
                        self.ping_pong_textures[!horizontal as usize]
                    };

                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, source_texture);
                    gl::Uniform1i(0, 0);
                    gl::Uniform1i(1, horizontal as GLint);
                    gl::DrawArrays(gl::TRIANGLES, 0, 3);

                    horizontal = !horizontal;
                }
                gl::BindVertexArray(0);
            }

            // reset frame buffer binding
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
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.ping_pong_textures[0]);
            gl::Uniform1i(0, 0);
            gl::Uniform1i(1, 1);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
        }
    }

    /// the current resolution
    #[inline]
    pub(crate) fn resolution(&self) -> (u32, u32) {
        (self.width as u32, self.height as u32)
    }
}

impl Drop for ScreenTexture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteTextures(1, &self.multi_texture);
            gl::DeleteTextures(1, &self.bloom_multi_texture);
            gl::DeleteRenderbuffers(1, &self.multi_rbo);
            gl::DeleteRenderbuffers(1, &self.rbo);
            gl::DeleteFramebuffers(1, &self.multi_fbo);
            gl::DeleteTextures(1, &self.texture);
            gl::DeleteTextures(1, &self.bloom_texture);
            gl::DeleteFramebuffers(1, &self.fbo);
            gl::DeleteFramebuffers(2, &self.ping_pong_fbos[0]);
            gl::DeleteTextures(2, &self.ping_pong_textures[0]);
        }
    }
}

/// Holds all parameters for post processing. This can be used to change the values of gamma, hue, saturation and brightness. For gamma, typical values are ``1.0`` (default) for linear color space and ``2.2`` for SRGB. The parameters of the HSV color space are all positive factors and are **not** absolute values. Hue is also in range [0, 1] inernally. You have to make shure the values are correct and valid yourself! The exposure parameter is an absolute value. The default value is 1.4. The ``use_bloom`` flag determines wether or not very bright parts of the scene should experience a bloom effect. This is turned on by default. There is also a ``bloom_threshold_shift`` setting that will control the brightness value at which bloom will be applied. This setting can also be controled for the skybox only. These values both default at 0. For bloom, you can also control the strength computation (iterations). The default value is 10 iterations. The ``background_as_scene_element`` flag determines wether or not the background clear color is used in the HDR color tone mapping of the engine. If this is set to true, the color will be affected by other settings like exposure and will be able to experience bloom. The default value is true.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PostProcessingParams {
    pub gamma: f32,
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub exposure: f32,
    pub use_bloom: bool,
    pub bloom_threshold_shift: f32,
    pub bloom_threshold_shift_skybox: f32,
    pub bloom_iterations: usize,
    pub background_as_scene_element: bool,
}

impl Default for PostProcessingParams {
    fn default() -> Self {
        Self {
            gamma: 1.0,
            hue: 1.0,
            saturation: 1.0,
            value: 1.0,
            exposure: 1.4,
            use_bloom: true,
            bloom_threshold_shift: 0.0,
            bloom_threshold_shift_skybox: 0.0,
            bloom_iterations: 10,
            background_as_scene_element: true,
        }
    }
}

/// stores all material data for the renderer
#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct MaterialData {
    pub(crate) ambient_color: Vec3,
    pub(crate) diffuse_color: Vec3,
    pub(crate) specular_color: Vec3,
    pub(crate) shininess: f32,
}
