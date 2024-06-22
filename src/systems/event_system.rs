use crate::ecs::component::{Acceleration, Velocity};
use crate::ecs::entity::EntityID;
use crate::utils::constants::INV_WIN_RATIO;
use crate::utils::threading::RefCountMutex;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::FullscreenType;

/// system managing the events
pub struct EventSystem {
    pub event_queue: SharedEventQueue,
    event_subsystem: sdl2::EventSubsystem,
    event_pump: sdl2::EventPump,
}

impl EventSystem {
    /// creates a new event system
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let event_subsystem = sdl_context.event().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        Self {
            event_queue: SharedEventQueue::init(),
            event_subsystem,
            event_pump,
        }
    }

    /// process all the sdl events in the event pump
    pub fn parse_sdl_events(&mut self, window: &mut sdl2::video::Window) -> Result<(), ()> {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    return Err(());
                }
                Event::KeyDown {
                    timestamp: _,
                    window_id: _,
                    keycode,
                    scancode: _,
                    keymod: _,
                    repeat: _,
                } => {
                    let key = keycode.unwrap();
                    if key == Keycode::F11 {
                        // toggle fullscreen
                        match window.fullscreen_state() {
                            FullscreenType::Off => {
                                window.set_fullscreen(FullscreenType::Desktop).unwrap();
                            }
                            FullscreenType::Desktop => {
                                window.set_fullscreen(FullscreenType::Off).unwrap();
                            }
                            _ => {
                                panic!("wrong fullscreen type detected");
                            }
                        }
                    }
                }
                Event::Window {
                    timestamp: _,
                    window_id: _,
                    win_event,
                } => {
                    if let WindowEvent::Resized(width, _) = win_event {
                        if !window.is_maximized()
                            && window.fullscreen_state() != FullscreenType::Desktop
                        {
                            window
                                .set_size(width as u32, (width as f32 * INV_WIN_RATIO) as u32)
                                .unwrap();
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

/// Events that can be processed by the event system
#[derive(Clone)]
pub enum PhysicsEvent {
    ChangeVelocity { e_id: EntityID, v: Velocity },
    ChangeAcceleration { e_id: EntityID, a: Acceleration },
}

/// threadsafe event queue
pub type SharedEventQueue = RefCountMutex<Vec<PhysicsEvent>>;

impl SharedEventQueue {
    /// creates a new queue
    pub fn init() -> Self {
        RefCountMutex::new(Vec::new())
    }

    /// adds an event to the queue
    pub fn push(&mut self, event: PhysicsEvent) {
        self.alter(|queue| {
            queue.push(event);
        });
    }

    /// clears the queue and yields all the stored events
    pub fn drain(&mut self) -> Vec<PhysicsEvent> {
        let mut events: Vec<PhysicsEvent> = vec![];
        self.alter(|queue| {
            events = queue.drain(..).collect();
        });
        events
    }
}
