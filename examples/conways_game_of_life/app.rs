use leafy::prelude::*;
use std::time::{Duration, Instant};

const GRID_SIZE: usize = 100;
const FPS: f64 = 10.0;

// custom entity flags
const IS_ALIVE: u64 = 50;
const WAS_ALIVE: u64 = 51;

pub struct App {
    grid_cells: Vec<Vec<EntityID>>,
    last_update: Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            grid_cells: vec![vec![NO_ENTITY; GRID_SIZE]; GRID_SIZE],
            last_update: Instant::now(),
        }
    }

    fn alive_neighbors(&self, row: usize, col: usize, engine: &Engine<Self>) -> usize {
        let mut count = 0;
        for i in row.saturating_sub(1)..=(row + 1).min(GRID_SIZE - 1) {
            for j in col.saturating_sub(1)..=(col + 1).min(GRID_SIZE - 1) {
                let nb = self.grid_cells[i][j];
                if !(i == row && j == col) && was_alive(nb, engine) {
                    count += 1;
                }
            }
        }
        count
    }
}

impl LeafyApp for App {
    fn init(&mut self, engine: &Engine<Self>) {
        let mut rendering_system = engine.rendering_system_mut();
        rendering_system.clear_color = Color32::TRANSPARENT;
        let sprite_grid = rendering_system.sprite_grid_mut(SpriteLayer::Layer0);
        sprite_grid.scale = 0.02;
        sprite_grid.center = Vec2::from_element(GRID_SIZE as f32 / 2.0 - 0.5);

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let (color, flags) = if row == GRID_SIZE / 2 - 1 && col == GRID_SIZE / 2
                    || row == GRID_SIZE / 2 && col == GRID_SIZE / 2 - 1
                    || row == GRID_SIZE / 2 && col == GRID_SIZE / 2
                    || row == GRID_SIZE / 2 + 1 && col == GRID_SIZE / 2
                    || row == GRID_SIZE / 2 + 1 && col == GRID_SIZE / 2 + 1
                {
                    (Color32::WHITE, EntityFlags::from_flags(&[WAS_ALIVE]))
                } else {
                    (Color32::TRANSPARENT, EntityFlags::default())
                };
                self.grid_cells[row][col] = engine.entity_manager_mut().create_entity(components!(
                    Sprite {
                        source: SpriteSource::Colored(color),
                        position: SpritePosition::Grid(vec2(col as f32, row as f32)),
                        layer: SpriteLayer::Layer0,
                    },
                    flags
                ));
            }
        }
    }

    fn on_frame_update(&mut self, engine: &Engine<Self>) {
        if self.last_update.elapsed() < Duration::from_secs_f64(1.0 / FPS) {
            return;
        }
        self.last_update = Instant::now();
        // update the alive flags
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let cell = self.grid_cells[row][col];
                let was_alive = was_alive(cell, engine);
                let neighbour_count = self.alive_neighbors(row, col, engine);
                let should_live = was_alive && (neighbour_count == 2 || neighbour_count == 3)
                    || !was_alive && neighbour_count == 3;
                set_is_alive(cell, should_live, engine);
            }
        }
        // update the colors and alive values of the past iteration
        for cell in self.grid_cells.iter().flatten().copied() {
            let is_alive = is_alive(cell, engine);
            let color = if is_alive {
                Color32::WHITE
            } else {
                Color32::TRANSPARENT
            };
            set_color(cell, color, engine);
            set_was_alive(cell, is_alive, engine);
        }
    }
}

fn is_alive<T: LeafyApp>(cell: EntityID, engine: &Engine<T>) -> bool {
    engine
        .entity_manager()
        .get_component::<EntityFlags>(cell)
        .unwrap()
        .get_bit(IS_ALIVE)
}

fn was_alive<T: LeafyApp>(cell: EntityID, engine: &Engine<T>) -> bool {
    engine
        .entity_manager()
        .get_component::<EntityFlags>(cell)
        .unwrap()
        .get_bit(WAS_ALIVE)
}

fn set_is_alive<T: LeafyApp>(cell: EntityID, alive: bool, engine: &Engine<T>) {
    engine
        .entity_manager_mut()
        .get_component_mut::<EntityFlags>(cell)
        .unwrap()
        .set_bit(IS_ALIVE, alive);
}

fn set_was_alive<T: LeafyApp>(cell: EntityID, alive: bool, engine: &Engine<T>) {
    engine
        .entity_manager_mut()
        .get_component_mut::<EntityFlags>(cell)
        .unwrap()
        .set_bit(WAS_ALIVE, alive);
}

fn set_color<T: LeafyApp>(cell: EntityID, color: Color32, engine: &Engine<T>) {
    engine
        .entity_manager_mut()
        .get_component_mut::<Sprite>(cell)
        .unwrap()
        .source = SpriteSource::Colored(color);
}
