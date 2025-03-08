use crate::engine::{Engine, FallingLeafApp};
use crate::systems::event_system::events::*;
use crate::utils::tools::{weak_ptr, SharedPtr, WeakPtr};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseScrollDelta, WindowEvent};
use winit::keyboard::PhysicalKey;

/// includes all of the requirements for a type to be used as an event
pub trait Event: Any + Debug {}
impl<T> Event for T where T: Any + Debug {}

/// system managing the events
pub struct EventSystem<A: FallingLeafApp> {
    phantom: PhantomData<A>,
    listeners: HashMap<TypeId, Vec<Box<dyn Any>>>,
    modifiers: HashMap<TypeId, Vec<Box<dyn Any>>>,
}

impl<A: FallingLeafApp> EventSystem<A> {
    /// creates a new event system
    pub(crate) fn new() -> Self {
        Self {
            phantom: PhantomData,
            listeners: HashMap::new(),
            modifiers: HashMap::new(),
        }
    }

    /// subscribe a handler to a specific event type
    pub fn add_listener<T: Event>(&mut self, handler: &SharedPtr<impl EventObserver<T> + 'static>) {
        let listeners = self.listeners.entry(TypeId::of::<T>()).or_default();
        listeners.push(Box::new(weak_ptr(handler) as WeakPtr<dyn EventObserver<T>>));
    }

    /// add a entity system modifier for a specific event type to the system
    pub fn add_modifier<T: Event>(&mut self, modifier: fn(&T, &Engine<A>)) {
        let wrapper = EventFunction { f: modifier };
        let modifiers = self.modifiers.entry(TypeId::of::<T>()).or_default();
        modifiers.push(Box::new(wrapper));
    }

    /// trigger an event and call all relevant functions/listeners
    pub(crate) fn trigger<T: Event>(&self, event: T, engine: &Engine<A>) {
        log::trace!("triggered event: {event:?}");
        if let Some(handlers) = self.listeners.get(&TypeId::of::<T>()) {
            for handler in handlers {
                let casted_handler = handler
                    .downcast_ref::<WeakPtr<dyn EventObserver<T>>>()
                    .unwrap();
                if let Some(handler_rc) = casted_handler.upgrade() {
                    let mut handler_ref = handler_rc.borrow_mut();
                    handler_ref.on_event(&event);
                }
            }
        }
        if let Some(modifiers) = self.modifiers.get(&TypeId::of::<T>()) {
            for modifier in modifiers {
                let casted = modifier.downcast_ref::<EventFunction<T, A>>().unwrap();
                (casted.f)(&event, engine);
            }
        }
    }

    /// process all the winit window events
    pub(crate) fn parse_winit_window_event(&self, event: WindowEvent, engine: &Engine<A>) {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic,
            } => match event.state {
                ElementState::Pressed => {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        self.trigger(
                            KeyPress {
                                key,
                                is_synthetic,
                                is_repeat: event.repeat,
                            },
                            engine,
                        );
                    }
                }
                ElementState::Released => {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        self.trigger(
                            KeyRelease {
                                key,
                                is_synthetic,
                                is_repeat: event.repeat,
                            },
                            engine,
                        );
                    }
                }
            },
            WindowEvent::Resized(size) => {
                self.trigger(
                    WindowResize {
                        width: size.width,
                        height: size.height,
                    },
                    engine,
                );
            }
            WindowEvent::Focused(gained) => {
                if gained {
                    self.trigger(WindowGainedFocus, engine);
                } else {
                    self.trigger(WindowLostFocus, engine);
                }
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => match state {
                ElementState::Pressed => {
                    self.trigger(MouseClick { button }, engine);
                }
                ElementState::Released => {
                    self.trigger(MouseRelease { button }, engine);
                }
            },
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.trigger(
                    MouseMove {
                        to_x: position.x,
                        to_y: position.y,
                    },
                    engine,
                );
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta: MouseScrollDelta::LineDelta(hori, vert),
                phase,
            } => {
                self.trigger(
                    MouseScroll {
                        vertical_lines: vert,
                        horizontal_lines: hori,
                        phase,
                    },
                    engine,
                );
            }
            WindowEvent::Moved(position) => {
                self.trigger(
                    WindowMoved {
                        to_x: position.x as u32,
                        to_y: position.y as u32,
                    },
                    engine,
                );
            }
            WindowEvent::DroppedFile(path) => {
                self.trigger(FileDropped { path }, engine);
            }
            WindowEvent::HoveredFile(path) => {
                self.trigger(FileHovered { path }, engine);
            }
            WindowEvent::HoveredFileCancelled => {
                self.trigger(FileHoverCancelled, engine);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.trigger(DPIScaleChange { scale_factor }, engine);
            }
            _ => (),
        }
    }

    /// process all the winit raw device events (e.g. for game controlls)
    pub(crate) fn parse_winit_device_event(
        &self,
        device_id: DeviceId,
        event: DeviceEvent,
        engine: &Engine<A>,
    ) {
        match event {
            DeviceEvent::Added => {
                self.trigger(RawDeviceAdded { device_id }, engine);
            }
            DeviceEvent::Removed => {
                self.trigger(RawDeviceRemoved { device_id }, engine);
            }
            DeviceEvent::MouseMotion { delta } => {
                self.trigger(
                    RawMouseMotion {
                        delta_x: delta.0,
                        delta_y: delta.1,
                    },
                    engine,
                );
            }
            DeviceEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(hori, vert),
            } => {
                self.trigger(
                    RawMouseScroll {
                        vertical_delta: vert,
                        horizontal_delta: hori,
                    },
                    engine,
                );
            }
            _ => (),
        }
    }
}

/// every struct that is supposed to be added to the event system as a listener has to implement this trait
/// for the specfic type of event it should listen to
pub trait EventObserver<T: Event>: Any {
    /// runs on every event trigger
    fn on_event(&mut self, event: &T);
}

/// holds the function pointer to the entity system event function
struct EventFunction<T: Event, A: FallingLeafApp> {
    pub(crate) f: fn(&T, &Engine<A>),
}

pub mod events {
    use std::path::PathBuf;
    use winit::event::{DeviceId, MouseButton, TouchPhase};
    use winit::keyboard::KeyCode;

    /// key press event data
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct KeyPress {
        pub key: KeyCode,
        pub is_synthetic: bool,
        pub is_repeat: bool,
    }

    /// key release event data
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct KeyRelease {
        pub key: KeyCode,
        pub is_synthetic: bool,
        pub is_repeat: bool,
    }

    /// mouse move event data (not for 3D camera control)
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct MouseMove {
        pub to_x: f64,
        pub to_y: f64,
    }

    /// mouse scroll event data
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct MouseScroll {
        pub vertical_lines: f32,
        pub horizontal_lines: f32,
        pub phase: TouchPhase,
    }

    /// mouse click event data
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct MouseClick {
        pub button: MouseButton,
    }

    /// mouse click event data
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct MouseRelease {
        pub button: MouseButton,
    }

    /// window resize event data (physical size)
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct WindowResize {
        pub width: u32,
        pub height: u32,
    }

    /// window focus lost event
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct WindowLostFocus;

    /// window focus gained event
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct WindowGainedFocus;

    /// window move event
    #[derive(Debug, Copy, Clone, PartialEq)]
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
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct FileHoverCancelled;

    /// DPI scale factor change event
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct DPIScaleChange {
        pub scale_factor: f64,
    }

    /// triggered if a device (might also be virtual from the OS) is added
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct RawDeviceAdded {
        pub device_id: DeviceId,
    }

    /// triggered if a device (might also be virtual from the OS) is removed
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct RawDeviceRemoved {
        pub device_id: DeviceId,
    }

    /// raw mouse move data (e.g. useful for game controls)
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct RawMouseMotion {
        pub delta_x: f64,
        pub delta_y: f64,
    }

    /// raw mouse scroll data (e.g. useful for game controls)
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct RawMouseScroll {
        pub vertical_delta: f32,
        pub horizontal_delta: f32,
    }

    /// contains all events that are also meant to be triggered by the user
    pub mod user_space {
        use crate::engine::EngineMode;
        use crate::glm;

        /// global change of the engine mode
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub struct EngineModeChange {
            pub new_mode: EngineMode,
        }

        /// Change of the users camera position, look and up direction vector used for rendering and audio processing. Changing the up axis to anything but the Y-axis will make the built-in mouse camera control useless and not work properly!
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub struct CamPositionChange {
            pub new_pos: glm::Vec3,
            pub new_look: glm::Vec3,
            pub new_up: glm::Vec3,
        }

        /// changes the animation speed of the rendering system
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub struct AnimationSpeedChange {
            pub new_animation_speed: f32,
        }
    }
}
