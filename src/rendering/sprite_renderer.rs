use crate::ecs::component::*;
use crate::ecs::entity_manager::EntityManager;
use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::mesh::Mesh;
use crate::rendering::shader::ShaderType;
use crate::utils::constants::bits::user_level::INVISIBLE;
use crate::utils::file::PLANE_MESH;
use gl::types::GLuint;
use stb_image::image::Image;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

/// identifier for a sprite sheet
pub type SpriteSheetID = u64;

/// renderer for 2D sprites
pub(crate) struct SpriteRenderer {
    renderer: BatchRenderer,
    plane_mesh: Mesh,
    sprite_sheets: HashMap<Rc<Path>, SpriteSheet>,
    sheet_transparency: HashMap<SpriteSheetSource, bool>,
    grid: SpriteGrid,
}

impl SpriteRenderer {
    /// creates a new sprite renderer
    pub(crate) fn new() -> Self {
        let plane_mesh = Mesh::from_bytes(PLANE_MESH);
        Self {
            renderer: BatchRenderer::new(&plane_mesh, ShaderType::Passthrough),
            plane_mesh,
            sprite_sheets: HashMap::new(),
            sheet_transparency: HashMap::new(),
            grid: SpriteGrid::new(10, 10),
        }
    }

    /// resets the renderer to the initial state
    pub(crate) fn reset(&mut self) {
        self.renderer.reset();
        self.renderer.clean_batches();
    }

    /// renders all sprites
    pub(crate) fn render(&self) {
        self.renderer.confirm_data();
        self.renderer.flush(None, ShaderType::Passthrough, false);
        unsafe { gl::DepthMask(gl::FALSE) };
        self.renderer.flush(None, ShaderType::Passthrough, true);
        unsafe { gl::DepthMask(gl::TRUE) };
    }

    /// adds the sprite data to the renderer
    pub(crate) fn add_data(&self, entity_manager: &EntityManager) {
        for (sprite, scale) in entity_manager
            .query3_opt2::<Sprite, Scale, EntityFlags>((None, None))
            .filter(|(_, _, f)| f.map_or(true, |flags| !flags.get_bit(INVISIBLE)))
            .map(|(p, s, _)| (p, s))
        {
            let scale = scale.copied().unwrap_or_default();
            todo!()
        }
    }
}

impl Drop for SpriteRenderer {
    fn drop(&mut self) {
        for tex_id in self.sprite_sheets.values().map(|sheet| sheet.texture_id) {
            unsafe { gl::DeleteTextures(1, &tex_id) };
        }
    }
}

/// source data for a sprite from a sprite sheet
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct SpriteSheetSource {
    path: Rc<Path>,
    pixel_index: (usize, usize),
    pixel_size: (usize, usize),
}

/// data associated with one sprite sheet
struct SpriteSheet {
    texture_id: GLuint,
    data: Image<u8>,
}

/// a sprite layout grid that can be used to position sprites
struct SpriteGrid {
    cells: Vec<Vec<u8>>,
}

impl SpriteGrid {
    fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![vec![0; height]; width],
        }
    }
}

// define layout grids and use the layout position
// -> layouts can be smoothly independantly moved, only render the part of the grid that is visible
// need way to influence the texture coodinates and not only read them from the mesh
// clean sprite sheets and maps if you dont need them anymore
// handle single sprite sources and the textures
