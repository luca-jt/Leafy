use crate::utils::tools::{weak_ptr, SharedPtr, WeakPtr};
use sdl2::controller::{Axis, Button};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::{MouseButton, MouseWheelDirection};
use std::any::{Any, TypeId};
use std::cell::RefMut;
use std::collections::HashMap;

/// system managing the events
pub struct EventSystem {
    event_subsystem: sdl2::EventSubsystem,
    event_pump: sdl2::EventPump,
    listeners: HashMap<TypeId, Vec<WeakPtr<dyn Any>>>,
}

impl EventSystem {
    /// creates a new event system
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let event_subsystem = sdl_context.event().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        Self {
            event_subsystem,
            event_pump,
            listeners: HashMap::new(),
        }
    }

    /// subscribe a handler to a specific event type
    pub fn add_listener<T: Any>(&mut self, handler: &SharedPtr<impl EventObserver<T> + 'static>) {
        let listeners = self
            .listeners
            .entry(TypeId::of::<T>())
            .or_insert(Vec::new());
        listeners.push(weak_ptr(handler) as WeakPtr<dyn Any>);
    }

    /// trigger an event
    pub fn trigger<T: Any>(&self, event: T) {
        if let Some(handlers) = self.listeners.get(&TypeId::of::<T>()) {
            for handler in handlers {
                if let Some(handler_rc) = handler.upgrade() {
                    let mut handler_ref = handler_rc.borrow_mut();
                    let casted_ref = handler_ref
                        .downcast_mut::<RefMut<dyn EventObserver<T>>>()
                        .unwrap();
                    casted_ref.on_event(&event);
                }
            }
        }
    }

    /// process all the sdl events in the event pump
    pub(crate) fn parse_sdl_events(&mut self) -> Result<(), ()> {
        let events: Vec<Event> = self.event_pump.poll_iter().collect();
        for event in events {
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
                        self.trigger(FLKeyPress { key });
                    }
                }
                Event::Window {
                    timestamp: _,
                    window_id: _,
                    win_event,
                } => {
                    if let WindowEvent::Resized(width, height) = win_event {
                        self.trigger(FLWindowResize { width, height });
                    } else if win_event == WindowEvent::FocusGained {
                        self.trigger(FLWindowGainedFocus);
                    } else if win_event == WindowEvent::FocusLost {
                        self.trigger(FLWindowLostFocus);
                    }
                }
                Event::ControllerButtonDown {
                    timestamp: _,
                    which: _,
                    button,
                } => {
                    self.trigger(FLControllerButton { button });
                }
                Event::ControllerAxisMotion {
                    timestamp: _,
                    which: _,
                    axis,
                    value,
                } => {
                    self.trigger(FLControllerAxis { axis, value });
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
                    self.trigger(FLMouseClick {
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
                    self.trigger(FLMouseMove {
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
                    self.trigger(FLMouseScroll {
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

        Ok(())
    }
}

pub trait EventObserver<T: Any> {
    /// runs on every event trigger
    fn on_event(&mut self, event: &T);
}

/// key press event data
pub struct FLKeyPress {
    pub key: Keycode,
}

/// mouse move event data
pub struct FLMouseMove {
    pub new_x: u32,
    pub new_y: u32,
    pub rel_x: u32,
    pub rel_y: u32,
}

/// mouse scroll event data
pub struct FLMouseScroll {
    pub direction: MouseWheelDirection,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub at_x: u32,
    pub at_y: u32,
}

/// mouse click event data
pub struct FLMouseClick {
    pub button: MouseButton,
    pub at_x: u32,
    pub at_y: u32,
}

/// controller axis event data
pub struct FLControllerAxis {
    pub axis: Axis,
    pub value: i16,
}

/// controller button event data
pub struct FLControllerButton {
    pub button: Button,
}

/// window resize event data
pub struct FLWindowResize {
    pub width: i32,
    pub height: i32,
}

/// window focus lost event
pub struct FLWindowLostFocus;

/// window focus gained event
pub struct FLWindowGainedFocus;
