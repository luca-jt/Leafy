use crate::ecs::component::{Acceleration, Velocity};
use crate::utils::threading::RefCountMutex;
use std::collections::VecDeque;

/// system managing the events
pub struct EventSystem {
    event_queue: RefCountMutex<VecDeque<Event>>,
}

impl EventSystem {
    /// creates a new event system
    pub fn new() -> Self {
        Self {
            event_queue: RefCountMutex::new(VecDeque::new()),
        }
    }
}

/// Events that can be processed by the event system
#[derive(Clone)]
pub enum Event {
    ChangeVelocity {
        entitiy: u64,
        velocity: Velocity,
    },
    ChangeAcceleration {
        entitiy: u64,
        acceleration: Acceleration,
    },
}
