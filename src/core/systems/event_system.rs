use sdl2::controller::{Axis, Button};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::{MouseButton, MouseWheelDirection};
use std::cell::RefCell;
use std::rc::Rc;

/// system managing the events
pub struct EventSystem {
    event_subsystem: sdl2::EventSubsystem,
    event_pump: sdl2::EventPump,
    listeners: Vec<Rc<RefCell<dyn EventObserver>>>,
}

impl EventSystem {
    /// creates a new event system
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let event_subsystem = sdl_context.event().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        Self {
            event_subsystem,
            event_pump,
            listeners: Vec::new(),
        }
    }

    /// subscribe a handler to a specific event type
    pub fn add_listener(&mut self, handler: Rc<RefCell<dyn EventObserver>>) {
        self.listeners.push(handler);
    }

    /// trigger an event
    pub fn trigger(&self, event: FLEvent) {
        for handler in self.listeners.iter() {
            handler.borrow_mut().on_event(&event);
        }
    }

    /// process all the sdl events in the event pump
    pub fn parse_sdl_events(&mut self) -> Result<(), ()> {
        let mut polled_events: Vec<FLEvent> = vec![];
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
                    if let Some(key) = keycode {
                        polled_events.push(FLEvent::KeyPress(key));
                    }
                }
                Event::Window {
                    timestamp: _,
                    window_id: _,
                    win_event,
                } => {
                    if let WindowEvent::Resized(width, height) = win_event {
                        polled_events.push(FLEvent::WindowResize(width, height))
                    } else if win_event == WindowEvent::FocusGained {
                        polled_events.push(FLEvent::WindowGainedFocus);
                    } else if win_event == WindowEvent::FocusLost {
                        polled_events.push(FLEvent::WindowLostFocus);
                    }
                }
                Event::ControllerButtonDown {
                    timestamp: _,
                    which: _,
                    button,
                } => {
                    polled_events.push(FLEvent::ControllerButton(button));
                }
                Event::ControllerAxisMotion {
                    timestamp: _,
                    which: _,
                    axis,
                    value,
                } => {
                    polled_events.push(FLEvent::ControllerAxis(axis, value));
                }
                Event::MouseButtonDown {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mouse_btn,
                    clicks: _,
                    x,
                    y,
                } => {
                    polled_events.push(FLEvent::MouseClick {
                        button: mouse_btn,
                        at_x: x as u32,
                        at_y: y as u32,
                    });
                }
                Event::MouseMotion {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mousestate: _,
                    x,
                    y,
                    xrel,
                    yrel,
                } => {
                    polled_events.push(FLEvent::MouseMove {
                        new_x: x as u32,
                        new_y: y as u32,
                        rel_x: xrel as u32,
                        rel_y: yrel as u32,
                    });
                }
                Event::MouseWheel {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    x: _,
                    y: _,
                    direction,
                    precise_x,
                    precise_y,
                    mouse_x,
                    mouse_y,
                } => {
                    polled_events.push(FLEvent::MouseScroll {
                        direction,
                        scroll_x: precise_x,
                        scroll_y: precise_y,
                        at_x: mouse_x as u32,
                        at_y: mouse_y as u32,
                    });
                }
                _ => {}
            }
        }
        // trigger all of the listeners
        for pe in polled_events {
            self.trigger(pe);
        }

        Ok(())
    }
}

pub trait EventObserver {
    /// runs on every event trigger
    fn on_event(&mut self, event: &FLEvent);
}

/// all of the supported event types for callbacks
pub enum FLEvent {
    KeyPress(Keycode),
    MouseMove {
        new_x: u32,
        new_y: u32,
        rel_x: u32,
        rel_y: u32,
    },
    MouseScroll {
        direction: MouseWheelDirection,
        scroll_x: f32,
        scroll_y: f32,
        at_x: u32,
        at_y: u32,
    },
    MouseClick {
        button: MouseButton,
        at_x: u32,
        at_y: u32,
    },
    ControllerAxis(Axis, i16),
    ControllerButton(Button),
    WindowResize(i32, i32),
    WindowLostFocus,
    WindowGainedFocus,
}

// Hashmap mit type ids?
// macro für die registry von events

/*
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
*/
