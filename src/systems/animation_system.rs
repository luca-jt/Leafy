use crate::systems::event_system::SharedEventQueue;

/// system managing the animations
pub struct AnimationSystem {
    event_queue: SharedEventQueue,
}

impl AnimationSystem {
    /// creates a new animation system
    pub fn new(event_queue: SharedEventQueue) -> Self {
        Self { event_queue }
    }

    /// updates the animations
    pub fn update(&mut self) {
        //...
    }
}

pub struct SampledAnimation {}
