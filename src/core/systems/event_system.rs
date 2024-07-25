use sdl2::controller::{Axis, Button};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::{MouseButton, MouseWheelDirection};
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// system managing the events
pub struct EventSystem {
    event_subsystem: sdl2::EventSubsystem,
    event_pump: sdl2::EventPump,
    listeners: HashMap<TypeId, Vec<Rc<RefCell<dyn EventObserver>>>>,
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
    pub fn add_listener<T: IFLEvent>(&mut self, handler: Rc<RefCell<dyn EventObserver>>) {
        let listeners = self
            .listeners
            .entry(TypeId::of::<T>())
            .or_insert(Vec::new());
        listeners.push(handler);
    }

    /// trigger an event
    pub fn trigger<T: IFLEvent + ?Sized>(&self, event: Box<T>) {
        if let Some(handlers) = self.listeners.get(&TypeId::of::<T>()) {
            let event_data = event.event_data();
            for handler in handlers {
                handler.borrow_mut().on_event(&event_data);
            }
        }
    }

    /// process all the sdl events in the event pump
    pub fn parse_sdl_events(&mut self) -> Result<(), ()> {
        let mut polled_events: Vec<Box<dyn IFLEvent>> = vec![];

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
                        polled_events.push(Box::new(FLKeyPress(key)));
                    }
                }
                Event::Window {
                    timestamp: _,
                    window_id: _,
                    win_event,
                } => {
                    if let WindowEvent::Resized(width, height) = win_event {
                        polled_events.push(Box::new(FLWindowResize(width, height)))
                    } else if win_event == WindowEvent::FocusGained {
                        polled_events.push(Box::new(FLWindowGainedFocus));
                    } else if win_event == WindowEvent::FocusLost {
                        polled_events.push(Box::new(FLWindowLostFocus));
                    }
                }
                Event::ControllerButtonDown {
                    timestamp: _,
                    which: _,
                    button,
                } => {
                    polled_events.push(Box::new(FLControllerButton(button)));
                }
                Event::ControllerAxisMotion {
                    timestamp: _,
                    which: _,
                    axis,
                    value,
                } => {
                    polled_events.push(Box::new(FLControllerAxis(axis, value)));
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
                    polled_events.push(Box::new(FLMouseClick {
                        button: mouse_btn,
                        at_x: x as u32,
                        at_y: y as u32,
                    }));
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
                    polled_events.push(Box::new(FLMouseMove {
                        new_x: x as u32,
                        new_y: y as u32,
                        rel_x: xrel as u32,
                        rel_y: yrel as u32,
                    }));
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
                    polled_events.push(Box::new(FLMouseScroll {
                        direction,
                        scroll_x: precise_x,
                        scroll_y: precise_y,
                        at_x: mouse_x as u32,
                        at_y: mouse_y as u32,
                    }));
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
    fn on_event(&mut self, event: &FLEventData);
}

pub trait IFLEvent: 'static {
    /// yields the data of the event
    fn event_data(&self) -> FLEventData;
}

/// all of the supported event types for callbacks
pub enum FLEventData {
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

pub struct FLKeyPress(Keycode);

impl IFLEvent for FLKeyPress {
    fn event_data(&self) -> FLEventData {
        FLEventData::KeyPress(self.0)
    }
}

pub struct FLMouseMove {
    new_x: u32,
    new_y: u32,
    rel_x: u32,
    rel_y: u32,
}

impl IFLEvent for FLMouseMove {
    fn event_data(&self) -> FLEventData {
        todo!()
    }
}

pub struct FLMouseScroll {
    direction: MouseWheelDirection,
    scroll_x: f32,
    scroll_y: f32,
    at_x: u32,
    at_y: u32,
}

impl IFLEvent for FLMouseScroll {
    fn event_data(&self) -> FLEventData {
        todo!()
    }
}

pub struct FLMouseClick {
    button: MouseButton,
    at_x: u32,
    at_y: u32,
}

impl IFLEvent for FLMouseClick {
    fn event_data(&self) -> FLEventData {
        todo!()
    }
}

pub struct FLControllerAxis(Axis, i16);

impl IFLEvent for FLControllerAxis {
    fn event_data(&self) -> FLEventData {
        todo!()
    }
}

pub struct FLControllerButton(Button);

impl IFLEvent for FLControllerButton {
    fn event_data(&self) -> FLEventData {
        todo!()
    }
}

pub struct FLWindowResize(i32, i32);

impl IFLEvent for FLWindowResize {
    fn event_data(&self) -> FLEventData {
        todo!()
    }
}

pub struct FLWindowLostFocus;

impl IFLEvent for FLWindowLostFocus {
    fn event_data(&self) -> FLEventData {
        todo!()
    }
}

pub struct FLWindowGainedFocus;

impl IFLEvent for FLWindowGainedFocus {
    fn event_data(&self) -> FLEventData {
        todo!()
    }
}
