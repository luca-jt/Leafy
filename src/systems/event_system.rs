use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
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
                    }
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
    MouseMove(f32, f32),
    MouseScroll(f32),
    MouseClick,
    WindowResize(i32, i32),
    WindowLooseFocus,
    WindowGainFocus,
}

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
