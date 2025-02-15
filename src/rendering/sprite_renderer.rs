use crate::ecs::component::utils::{Color32, SpriteLayer, SpritePosition, SpriteSource};
use crate::ecs::component::*;
use crate::ecs::entity_manager::EntityManager;
use crate::glm;
use crate::rendering::shader::bind_sprite_attribs;
use crate::utils::constants::bits::user_level::INVISIBLE;
use crate::utils::constants::MAX_TEXTURE_COUNT;
use crate::utils::file::{SPRITE_PLANE_INDICES, SPRITE_PLANE_UVS, SPRITE_PLANE_VERTICES};
use crate::utils::tools::mult_mat4_vec3;
use gl::types::*;
use std::collections::HashSet;
use std::ptr;

const PLANE_MESH_NUM_VERTICES: usize = 4;
const PLANE_MESH_NUM_INDICES: usize = 6;

/// renderer for 2D sprites
pub(crate) struct SpriteRenderer {
    renderer_map: [Vec<SpriteBatch>; 10],
    used_batch_indices: [HashSet<usize>; 10],
    white_texture: GLuint,
    samplers: [GLint; MAX_TEXTURE_COUNT],
    pub(crate) grids: [SpriteGrid; 10],
}

impl SpriteRenderer {
    /// creates a new sprite renderer
    pub(crate) fn new() -> Self {
        let mut white_texture = 0;
        unsafe {
            // 1x1 WHITE TEXTURE
            gl::GenTextures(1, &mut white_texture);
            gl::BindTexture(gl::TEXTURE_2D, white_texture);
            let white_color_data: Vec<u8> = vec![255, 255, 255, 255];
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as GLint,
                1,
                1,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                white_color_data.as_ptr() as *const GLvoid,
            );
        }
        // TEXTURE SAMPLERS
        let mut samplers: [GLint; MAX_TEXTURE_COUNT] = [0; MAX_TEXTURE_COUNT];
        for (i, sampler) in samplers.iter_mut().enumerate() {
            *sampler = i as GLint;
        }

        Self {
            renderer_map: Default::default(),
            used_batch_indices: Default::default(),
            white_texture,
            samplers,
            grids: [SpriteGrid::default(); 10],
        }
    }

    /// resets the renderer to the initial state
    pub(crate) fn reset(&mut self) {
        for batch in self.renderer_map.iter_mut().flatten() {
            batch.reset();
        }
        for (layer, batches) in self.renderer_map.iter_mut().enumerate() {
            for index in 0..batches.len() {
                if !self.used_batch_indices[layer].contains(&index) {
                    batches.remove(index);
                }
            }
            self.used_batch_indices[layer].clear();
        }
    }

    /// renders all sprites
    pub(crate) fn render(&self) {
        for batch in self.renderer_map.iter().flatten() {
            batch.confirm_data();
        }
        unsafe {
            // bind uniforms
            gl::Uniform1iv(7, MAX_TEXTURE_COUNT as GLsizei, &self.samplers[0]);
            // bind texture
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.white_texture);
        }
        for layer in self.renderer_map.iter().rev() {
            for batch in layer {
                batch.flush();
            }
        }
    }

    /// adds the sprite data to the renderer
    pub(crate) fn add_data(&mut self, entity_manager: &EntityManager) {
        for (sprite, scale) in unsafe {
            entity_manager.query3::<&Sprite, Option<&Scale>, Option<&EntityFlags>>((None, None))
        }
        .filter(|(_, _, f)| f.map_or(true, |flags| !flags.get_bit(INVISIBLE)))
        .filter(|(s, _, _)| match s.source {
            SpriteSource::Colored(color) => color != Color32::TRANSPARENT,
            _ => true,
        })
        .map(|(p, s, _)| (p, s))
        {
            let scale = scale.copied().unwrap_or_default().scale_matrix();
            let trafo = match sprite.position {
                SpritePosition::Grid(pos) => {
                    let grid = self.grids[sprite.layer as usize];
                    let abs_pos = (pos - grid.center) * grid.scale;
                    let position = glm::vec3(abs_pos.x, abs_pos.y, sprite.layer.to_z_coord());
                    &(glm::translate(&glm::Mat4::identity(), &position)
                        * scale
                        * Scale::from_factor(grid.scale).scale_matrix())
                }
                SpritePosition::Absolute(abs_pos) => {
                    let position = glm::vec3(abs_pos.x, abs_pos.y, sprite.layer.to_z_coord());
                    &(glm::translate(&glm::Mat4::identity(), &position) * scale)
                }
            };
            match &sprite.source {
                SpriteSource::Sheet(src) => {
                    let sheet = entity_manager
                        .sprite_texture_map
                        .get_sheet(&src.path)
                        .unwrap();
                    let mut tex_coords = SPRITE_PLANE_UVS;
                    for coord in tex_coords.iter_mut() {
                        *coord = coord.component_mul(&glm::vec2(
                            src.pixel_size.0 as f32 / sheet.width as f32,
                            src.pixel_size.1 as f32 / sheet.height as f32,
                        )) + glm::vec2(
                            src.pixel_index.0 as f32 / sheet.width as f32,
                            src.pixel_index.1 as f32 / sheet.height as f32,
                        );
                    }
                    let config = SpriteConfig {
                        tex_id: sheet.texture_id,
                        tex_coords,
                        layer: sprite.layer,
                        trafo,
                    };
                    self.add_tex_sprite(config);
                }
                SpriteSource::Colored(color) => {
                    self.add_color_sprite(*color, sprite.layer, trafo);
                }
                SpriteSource::Single(path) => {
                    let tex_id = entity_manager
                        .sprite_texture_map
                        .get_sprite_id(path)
                        .unwrap();
                    let config = SpriteConfig {
                        tex_id,
                        tex_coords: SPRITE_PLANE_UVS,
                        layer: sprite.layer,
                        trafo,
                    };
                    self.add_tex_sprite(config);
                }
            }
        }
    }

    /// adds a sprite with a plain color
    pub(crate) fn add_color_sprite(
        &mut self,
        color: Color32,
        layer: SpriteLayer,
        trafo: &glm::Mat4,
    ) {
        self.used_batch_indices[layer as usize].insert(0);
        if self.renderer_map[layer as usize].is_empty() {
            self.renderer_map[layer as usize].push(SpriteBatch::new());
        }
        self.renderer_map[layer as usize]
            .first_mut()
            .unwrap()
            .add_color_sprite(trafo, color);
    }

    /// adds a sprite with a texture
    pub(crate) fn add_tex_sprite(&mut self, config: SpriteConfig) {
        if self.renderer_map[config.layer as usize].is_empty() {
            self.renderer_map[config.layer as usize].push(SpriteBatch::new());
        }
        for (i, batch) in self.renderer_map[config.layer as usize]
            .iter_mut()
            .enumerate()
        {
            if batch.try_add_tex_sprite(&config) {
                self.used_batch_indices[config.layer as usize].insert(i);
                return;
            }
        }
        // add new batch
        let mut new_batch = SpriteBatch::new();
        let res = new_batch.try_add_tex_sprite(&config);
        debug_assert!(res);
        self.used_batch_indices[config.layer as usize]
            .insert(self.renderer_map[config.layer as usize].len());
        self.renderer_map[config.layer as usize].push(new_batch);
    }
}

impl Drop for SpriteRenderer {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.white_texture) };
    }
}

/// a sprite render batch
struct SpriteBatch {
    vao: GLuint,
    vbo: GLuint,
    ibo: GLuint,
    index_count: GLsizei,
    obj_buffer: Vec<SpriteVertex>,
    obj_buffer_ptr: usize,
    all_tex_ids: Vec<GLuint>,
    max_num_meshes: usize,
}

impl SpriteBatch {
    /// creates a new batch with default size
    fn new() -> Self {
        let max_num_meshes: usize = 10;
        let obj_buffer = vec![SpriteVertex::default(); PLANE_MESH_NUM_VERTICES * max_num_meshes];
        let mut vao = 0;
        let mut vbo = 0;
        let mut ibo = 0;
        unsafe {
            // GENERATE BUFFERS
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::CreateBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (PLANE_MESH_NUM_VERTICES * max_num_meshes * size_of::<SpriteVertex>())
                    as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            // BIND ATTRIB POINTERS
            bind_sprite_attribs();

            // INDICES
            let mut indices: Vec<GLuint> = vec![0; PLANE_MESH_NUM_INDICES * max_num_meshes];
            for i in 0..PLANE_MESH_NUM_INDICES * max_num_meshes {
                indices[i] = SPRITE_PLANE_INDICES[i % PLANE_MESH_NUM_INDICES]
                    + PLANE_MESH_NUM_VERTICES as GLuint
                        * (i as GLuint / PLANE_MESH_NUM_INDICES as GLuint);
            }
            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (PLANE_MESH_NUM_INDICES * max_num_meshes * size_of::<GLuint>()) as GLsizeiptr,
                indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            gl::BindVertexArray(0);
        }
        log::debug!("new sprite batch created");

        Self {
            vao,
            vbo,
            ibo,
            index_count: 0,
            obj_buffer,
            obj_buffer_ptr: 0,
            all_tex_ids: Vec::new(),
            max_num_meshes,
        }
    }

    /// resize the buffer for more mesh data
    fn resize_buffer(&mut self) {
        let add_size: usize = self.max_num_meshes * 2;
        self.max_num_meshes += add_size;
        self.obj_buffer.reserve_exact(add_size);
        self.obj_buffer.extend(vec![
            SpriteVertex::default();
            PLANE_MESH_NUM_VERTICES * add_size
        ]);
        log::debug!("resized sprite batch to: {:?}", self.max_num_meshes);
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (PLANE_MESH_NUM_VERTICES * self.max_num_meshes * size_of::<SpriteVertex>())
                    as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            let mut indices: Vec<GLuint> = vec![0; PLANE_MESH_NUM_INDICES * self.max_num_meshes];
            for i in 0..PLANE_MESH_NUM_INDICES * self.max_num_meshes {
                indices[i] = SPRITE_PLANE_INDICES[i % PLANE_MESH_NUM_INDICES]
                    + PLANE_MESH_NUM_VERTICES as GLuint
                        * (i as GLuint / PLANE_MESH_NUM_INDICES as GLuint);
            }
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (PLANE_MESH_NUM_INDICES * self.max_num_meshes * size_of::<GLuint>()) as GLsizeiptr,
                indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
        }
    }

    /// end the render batch
    fn confirm_data(&self) {
        // dynamically copy the the drawn mesh vertex data from object buffer into the vertex buffer on the gpu
        unsafe {
            let vertices_size: GLsizeiptr =
                (self.obj_buffer_ptr * size_of::<SpriteVertex>()) as GLsizeiptr;
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                vertices_size,
                self.obj_buffer.as_ptr() as *const GLvoid,
            );
        }
    }

    /// flushes this batch and renders the stored geometry
    fn flush(&self) {
        unsafe {
            // bind textures
            for (unit, tex_id) in self.all_tex_ids.iter().enumerate() {
                gl::ActiveTexture(gl::TEXTURE1 + unit as GLenum);
                gl::BindTexture(gl::TEXTURE_2D, *tex_id);
            }
            // draw the triangles corresponding to the index buffer
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl::BindVertexArray(0);
        }
    }

    /// resets the batch to the initial state
    fn reset(&mut self) {
        self.index_count = 0;
        self.obj_buffer_ptr = 0;
    }

    /// adds a sprite with a texture to the batch
    fn try_add_tex_sprite(&mut self, config: &SpriteConfig) -> bool {
        // determine texture index
        let mut tex_index: GLfloat = -1.0;
        for (i, id) in self.all_tex_ids.iter().enumerate() {
            if *id == config.tex_id {
                tex_index = (i + 1) as GLfloat;
                break;
            }
        }
        if tex_index == -1.0 {
            if self.all_tex_ids.len() >= MAX_TEXTURE_COUNT - 1 {
                // start a new batch if out of texture slots
                return false;
            }
            tex_index = (self.all_tex_ids.len() + 1) as GLfloat;
            self.all_tex_ids.push(config.tex_id);
        }
        if self.index_count as usize >= PLANE_MESH_NUM_INDICES * self.max_num_meshes {
            // resize current batch if batch size exceeded
            self.resize_buffer();
        }
        // copy mesh vertex data into the object buffer
        for i in 0..PLANE_MESH_NUM_VERTICES {
            *self.obj_buffer.get_mut(self.obj_buffer_ptr).unwrap() = SpriteVertex {
                position: mult_mat4_vec3(config.trafo, &SPRITE_PLANE_VERTICES[i]),
                color: glm::vec4(1.0, 1.0, 1.0, 1.0),
                uv_coords: config.tex_coords[i],
                tex_index,
            };
            self.obj_buffer_ptr += 1;
        }
        self.index_count += PLANE_MESH_NUM_INDICES as GLsizei;
        true
    }

    /// adds a sprite with a color to the batch
    fn add_color_sprite(&mut self, trafo: &glm::Mat4, color: Color32) {
        if self.index_count as usize >= PLANE_MESH_NUM_INDICES * self.max_num_meshes {
            // resize current batch if batch size exceeded
            self.resize_buffer();
        }

        let tex_index: GLfloat = 0.0; // white texture

        // copy mesh vertex data into the object buffer
        for i in 0..PLANE_MESH_NUM_VERTICES {
            *self.obj_buffer.get_mut(self.obj_buffer_ptr).unwrap() = SpriteVertex {
                position: mult_mat4_vec3(trafo, &SPRITE_PLANE_VERTICES[i]),
                color: color.to_vec4(),
                uv_coords: SPRITE_PLANE_UVS[i],
                tex_index,
            };
            self.obj_buffer_ptr += 1;
        }
        self.index_count += PLANE_MESH_NUM_INDICES as GLsizei;
    }
}

impl Drop for SpriteBatch {
    fn drop(&mut self) {
        log::debug!("dropped sprite batch");
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ibo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

/// data for a single vertex
#[derive(Default, Clone, Copy, Debug)]
#[repr(C)]
pub(crate) struct SpriteVertex {
    pub(crate) position: glm::Vec3,
    pub(crate) color: glm::Vec4,
    pub(crate) uv_coords: glm::Vec2,
    pub(crate) tex_index: GLfloat,
}

/// the sprite render config for the renderer
pub(crate) struct SpriteConfig<'a> {
    pub(crate) tex_id: GLuint,
    pub(crate) tex_coords: [glm::Vec2; 4],
    pub(crate) layer: SpriteLayer,
    pub(crate) trafo: &'a glm::Mat4,
}

/// data associated with one sprite sheet
pub(crate) struct SpriteSheet {
    pub(crate) texture_id: GLuint,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

/// config data for a sprite grid (default is scale 1 and center origin)
#[derive(Debug, Copy, Clone)]
pub struct SpriteGrid {
    pub scale: f32,
    pub center: glm::Vec2,
}

impl Default for SpriteGrid {
    fn default() -> Self {
        Self {
            scale: 1.0,
            center: glm::vec2(0.0, 0.0),
        }
    }
}
