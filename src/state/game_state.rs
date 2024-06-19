use crate::ecs::entity_manager::EntityManager;

/// state for the game logic
pub struct GameState {
    entity_manager: EntityManager,
}

impl GameState {
    /// creates a new game state
    pub fn new() -> Self {
        Self {
            entity_manager: EntityManager::new(),
        }
    }

    /// updates the current game state
    pub fn update(&mut self) {
        //...
    }
}
