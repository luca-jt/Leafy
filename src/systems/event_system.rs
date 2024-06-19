use crate::ecs::component::{Acceleration, Velocity};
use crate::ecs::entity::EntityID;
use crate::utils::threading::RefCountMutex;

/// system managing the events
pub struct EventSystem {
    event_queue: EventQueue,
    event_subsystem: sdl2::EventSubsystem,
    event_pump: sdl2::EventPump,
}

impl EventSystem {
    /// creates a new event system
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let event_subsystem = sdl_context.event().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        Self {
            event_queue: EventQueue::init(),
            event_subsystem,
            event_pump,
        }
    }
}

/// Events that can be processed by the event system
#[derive(Clone)]
pub enum Event {
    ChangeVelocity {
        entitiy: EntityID,
        velocity: Velocity,
    },
    ChangeAcceleration {
        entitiy: EntityID,
        acceleration: Acceleration,
    },
}

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
