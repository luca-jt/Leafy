use crate::ecs::asset_manager::AssetManager;
use crate::ecs::component::{Acceleration, Position, Velocity};
use crate::ecs::entity::{Entity, EntityType};
use crate::ecs::entity_manager::EntityManager;
use crate::rendering::mesh::Mesh;

/// state for the game logic
pub struct GameState {
    entity_manager: EntityManager,
    asset_manager: AssetManager,
}

impl GameState {
    /// creates a new game state
    pub fn new() -> Self {
        let mut entity_manager = EntityManager::new();
        let mut asset_manager = AssetManager::new();
        let sphere_id = asset_manager.add_asset(Mesh::new("sphere.obj"));

        let test_entity = Entity {
            t: EntityType::Sphere(sphere_id),
            position: Position::zeros(),
            velocity: Velocity::zeros(),
            acceleration: Acceleration::zeros(),
        };
        let _ = entity_manager.add_entity(test_entity);

        Self {
            entity_manager,
            asset_manager,
        }
    }

    /// updates the current game state
    pub fn update(&mut self) {
        //...
    }
}
