use crate::ecs::component::{Acceleration, Velocity};
use crate::ecs::entity::EntityID;
use crate::state::game_state::GameState;
use crate::utils::constants::INV_WIN_RATIO;
use crate::utils::threading::RefCountMutex;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::FullscreenType;

/// system managing the events
pub struct EventSystem {
    pub physics_event_queue: SharedEventQueue<PhysicsEvent>,
    pub key_event_queue: Vec<KeyPressEvent>,
    event_subsystem: sdl2::EventSubsystem,
    event_pump: sdl2::EventPump,
}

impl EventSystem {
    /// creates a new event system
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let event_subsystem = sdl_context.event().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        Self {
            physics_event_queue: SharedEventQueue::init(),
            key_event_queue: Self::create_key_events(),
            event_subsystem,
            event_pump,
        }
    }

    /// adds key press events to the queue
    fn create_key_events() -> Vec<KeyPressEvent> {
        // ...
        vec![]
    }

    /// process all the sdl events in the event pump
    pub fn parse_sdl_events(
        &mut self,
        window: &mut sdl2::video::Window,
        game_state: &mut GameState,
    ) -> Result<(), ()> {
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
                    let key = keycode.unwrap_or(Keycode::AMPERSAND); // just smth random that nobody uses
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
                    for key_event in self.key_event_queue.iter_mut() {
                        if key == key_event.key {
                            let f = &mut key_event.callback;
                            f(game_state);
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

/// phyics events for entities
#[derive(Clone)]
pub enum PhysicsEvent {
    ChangeVelocity { e_id: EntityID, v: Velocity },
    ChangeAcceleration { e_id: EntityID, a: Acceleration },
}

/// key press events
pub struct KeyPressEvent {
    pub key: Keycode,
    pub callback: Box<dyn FnMut(&mut GameState)>,
}

/// threadsafe event queue
pub type SharedEventQueue<T> = RefCountMutex<Vec<T>>;

impl<T: Clone> SharedEventQueue<T> {
    /// creates a new queue
    pub fn init() -> Self {
        RefCountMutex::new(Vec::new())
    }

    /// adds an event to the queue
    pub fn push(&mut self, event: T) {
        self.alter(|queue| {
            queue.push(event);
        });
    }

    /// clears the queue and yields all the stored events
    pub fn drain(&mut self) -> Vec<T> {
        let mut events: Vec<T> = vec![];
        self.alter(|queue| {
            events = queue.drain(..).collect();
        });
        events
    }
}
