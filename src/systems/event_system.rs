use crate::ecs::component::{Acceleration, Velocity};
use crate::utils::threading::RefCountMutex;

pub type EventQueue = RefCountMutex<Vec<Event>>;

impl EventQueue {
    /// creates a new queue
    pub fn init() -> Self {
        RefCountMutex::new(Vec::new())
    }

    /// adds an event to the queue
    pub fn push(&mut self, event: Event) {
        self.alter(|queue| {
            queue.push(event);
        });
    }

    /// clears the queue and yields all the stored events
    pub fn drain(&mut self) -> Vec<Event> {
        let mut events: Vec<Event> = vec![];
        self.alter(|queue| {
            events = queue.drain(..).collect();
        });
        events
    }
}

/// system managing the events
pub struct EventSystem {
    event_queue: EventQueue,
}

impl EventSystem {
    /// creates a new event system
    pub fn new() -> Self {
        Self {
            event_queue: EventQueue::init(),
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
