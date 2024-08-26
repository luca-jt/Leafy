use std::any::{Any, TypeId};
use std::cell::RefMut;
use std::collections::HashMap;

use winit::event::{DeviceEvent, DeviceId, ElementState, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;

use crate::systems::event_system::events::*;
use crate::utils::tools::{weak_ptr, SharedPtr, WeakPtr};

/// system managing the events
pub struct EventSystem {
    pub(crate) event_loop: Option<EventLoop<()>>,
    listeners: HashMap<TypeId, Vec<WeakPtr<dyn Any>>>,
}

impl EventSystem {
    /// creates a new event system
    pub(crate) fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        Self {
            event_loop: Some(event_loop),
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

    /// process all the winit window events
    pub(crate) fn parse_winit_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic,
            } => match event.state {
                ElementState::Pressed => {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        self.trigger(KeyPress {
                            key,
                            is_synthetic,
                            is_repeat: event.repeat,
                        });
                    }
                }
                ElementState::Released => {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        self.trigger(KeyRelease {
                            key,
                            is_synthetic,
                            is_repeat: event.repeat,
                        });
                    }
                }
            },
            WindowEvent::Resized(size) => {
                self.trigger(WindowResize {
                    width: size.width,
                    height: size.height,
                });
            }
            WindowEvent::Focused(gained) => {
                if gained {
                    self.trigger(WindowGainedFocus);
                } else {
                    self.trigger(WindowLostFocus);
                }
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => match state {
                ElementState::Pressed => {
                    self.trigger(MouseClick { button });
                }
                ElementState::Released => {
                    self.trigger(MouseRelease { button });
                }
            },
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.trigger(MouseMove {
                    to_x: position.x,
                    to_y: position.y,
                });
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase,
            } => {
                if let MouseScrollDelta::LineDelta(hori, vert) = delta {
                    self.trigger(MouseScroll {
                        vertical_lines: vert,
                        horizontal_lines: hori,
                        phase,
                    });
                }
            }
            WindowEvent::Moved(position) => {
                self.trigger(WindowMoved {
                    to_x: position.x as u32,
                    to_y: position.y as u32,
                });
            }
            WindowEvent::DroppedFile(path) => {
                self.trigger(FileDropped { path });
            }
            WindowEvent::HoveredFile(path) => {
                self.trigger(FileHovered { path });
            }
            WindowEvent::HoveredFileCancelled => {
                self.trigger(FileHoverCancelled);
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer,
            } => {
                self.trigger(DPIScaleFactorChanged {
                    scale_factor,
                    size_writer: inner_size_writer,
                });
            }
            _ => (),
        }
    }

    /// process all the winit raw device events (e.g. for game controlls)
    pub(crate) fn parse_winit_device_event(&mut self, device_id: DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::Added => {
                self.trigger(RawDeviceAdded { device_id });
            }
            DeviceEvent::Removed => {
                self.trigger(RawDeviceRemoved { device_id });
            }
            DeviceEvent::MouseMotion { delta } => {
                self.trigger(RawMouseMotion {
                    delta_x: delta.0,
                    delta_y: delta.1,
                });
            }
            DeviceEvent::MouseWheel { delta } => {
                if let MouseScrollDelta::LineDelta(hori, vert) = delta {
                    self.trigger(RawMouseScroll {
                        vertical_delta: vert,
                        horizontal_delta: hori,
                    });
                }
            }
            _ => (),
        }
    }
}

/// every struct that is supposed to be added to the event system as a listener has to implement this trait
/// for the specfic type of event it should listen to
pub trait EventObserver<T: Any> {
    /// runs on every event trigger
    fn on_event(&mut self, event: &T);
}

pub mod events {
    use std::path::PathBuf;
    use winit::event::{DeviceId, InnerSizeWriter, MouseButton, TouchPhase};
    use winit::keyboard::KeyCode;

    /// key press event data
    #[derive(Debug, Clone, PartialEq)]
    pub struct KeyPress {
        pub key: KeyCode,
        pub is_synthetic: bool,
        pub is_repeat: bool,
    }

    /// key release event data
    #[derive(Debug, Clone, PartialEq)]
    pub struct KeyRelease {
        pub key: KeyCode,
        pub is_synthetic: bool,
        pub is_repeat: bool,
    }

    /// mouse move event data (not for 3D camera control)
    #[derive(Debug, Clone, PartialEq)]
    pub struct MouseMove {
        pub to_x: f64,
        pub to_y: f64,
    }

    /// mouse scroll event data
    #[derive(Debug, Clone, PartialEq)]
    pub struct MouseScroll {
        pub vertical_lines: f32,
        pub horizontal_lines: f32,
        pub phase: TouchPhase,
    }

    /// mouse click event data
    #[derive(Debug, Clone, PartialEq)]
    pub struct MouseClick {
        pub button: MouseButton,
    }

    /// mouse click event data
    #[derive(Debug, Clone, PartialEq)]
    pub struct MouseRelease {
        pub button: MouseButton,
    }

    /// window resize event data
    #[derive(Debug, Clone, PartialEq)]
    pub struct WindowResize {
        pub width: u32,
        pub height: u32,
    }

    /// window focus lost event
    #[derive(Debug, Clone, PartialEq)]
    pub struct WindowLostFocus;

    /// window focus gained event
    #[derive(Debug, Clone, PartialEq)]
    pub struct WindowGainedFocus;

    /// window move event
    #[derive(Debug, Clone, PartialEq)]
    pub struct WindowMoved {
        pub to_x: u32,
        pub to_y: u32,
    }

    /// file drop event
    #[derive(Debug, Clone, PartialEq)]
    pub struct FileDropped {
        pub path: PathBuf,
    }

    /// file hover event
    #[derive(Debug, Clone, PartialEq)]
    pub struct FileHovered {
        pub path: PathBuf,
    }

    /// file hover cancel event
    #[derive(Debug, Clone, PartialEq)]
    pub struct FileHoverCancelled;

    /// DPI scale factor change event
    #[derive(Debug, Clone, PartialEq)]
    pub struct DPIScaleFactorChanged {
        pub scale_factor: f64,
        pub size_writer: InnerSizeWriter,
    }

    /// triggered if a device (might also be virtual from the OS) is added
    #[derive(Debug, Clone, PartialEq)]
    pub struct RawDeviceAdded {
        pub device_id: DeviceId,
    }

    /// triggered if a device (might also be virtual from the OS) is removed
    #[derive(Debug, Clone, PartialEq)]
    pub struct RawDeviceRemoved {
        pub device_id: DeviceId,
    }

    /// raw mouse move data (e.g. useful for game controls)
    #[derive(Debug, Clone, PartialEq)]
    pub struct RawMouseMotion {
        pub delta_x: f64,
        pub delta_y: f64,
    }

    /// raw mouse scroll data (e.g. useful for game controls)
    #[derive(Debug, Clone, PartialEq)]
    pub struct RawMouseScroll {
        pub vertical_delta: f32,
        pub horizontal_delta: f32,
    }
}
