use falling_leaf::ecs::component::utils::*;
use falling_leaf::ecs::component::Sprite;
use falling_leaf::ecs::entity::EntityID;
use falling_leaf::engine::{Engine, FallingLeafApp};
use falling_leaf::utils::constants::NO_ENTITY;
use falling_leaf::{components, glm};

const GRID_SIZE: usize = 120;

pub struct App {
    grid_cells: Vec<Vec<EntityID>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            grid_cells: vec![vec![NO_ENTITY; GRID_SIZE]; GRID_SIZE],
        }
    }
}

impl FallingLeafApp for App {
    fn init(&mut self, engine: &Engine<Self>) {
        engine
            .rendering_system_mut()
            .set_gl_clearcolor(Color32::TRANSPARENT);

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                self.grid_cells[row][col] =
                    engine
                        .entity_manager_mut()
                        .create_entity(components!(Sprite {
                            source: SpriteSource::Colored(Color32::BLACK),
                            position: SpritePosition::Grid(glm::vec2(col as f32, row as f32)),
                            layer: SpriteLayer::Layer0,
                        }));
            }
        }

        // TODO: set the middle of the cells to this pattern:
        // #
        //##
        // ##
    }

    fn on_frame_update(&mut self, _engine: &Engine<Self>) {}
}
