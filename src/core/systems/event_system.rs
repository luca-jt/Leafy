use crate::ecs::entity_manager::EntityManager;
use crate::engine::FallingLeafApp;
use std::any::{Any, TypeId};
use std::cell::RefMut;
use std::collections::HashMap;
use std::ops::DerefMut;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseScrollDelta, WindowEvent};
use winit::keyboard::PhysicalKey;

use crate::systems::event_system::events::*;
use crate::utils::tools::{weak_ptr, SharedPtr, WeakPtr};

/// system managing the events
pub struct EventSystem {
    pub(crate) app: Option<WeakPtr<Box<dyn FallingLeafApp>>>,
    entity_manager: WeakPtr<EntityManager>,
    listeners: HashMap<TypeId, Vec<Box<dyn Any>>>,
    entity_modifiers: HashMap<TypeId, Vec<Box<dyn Any>>>,
}

impl EventSystem {
    /// creates a new event system
    pub(crate) fn new(entity_manager: &SharedPtr<EntityManager>) -> Self {
        Self {
            app: None,
            entity_manager: weak_ptr(entity_manager),
            listeners: HashMap::new(),
            entity_modifiers: HashMap::new(),
        }
    }

    /// subscribe a handler to a specific event type
    pub fn add_listener<T: Any>(&mut self, handler: &SharedPtr<impl EventObserver<T> + 'static>) {
        let listeners = self.listeners.entry(TypeId::of::<T>()).or_default();
        listeners.push(Box::new(weak_ptr(handler) as WeakPtr<dyn EventObserver<T>>));
    }

    /// add a entity system modifier for a specific event type to the system
    pub fn add_modifier<T: Any>(
        &mut self,
        modifier: fn(&T, RefMut<Box<dyn FallingLeafApp>>, &mut EntityManager),
    ) {
        let wrapper = EventFunction { f: modifier };
        let modifiers = self.entity_modifiers.entry(TypeId::of::<T>()).or_default();
        modifiers.push(Box::new(wrapper));
    }

    /// trigger an event and call all relevant functions of listeners
    pub fn trigger<T: Any>(&self, event: T) {
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
        if let (Some(modifiers), Some(weak_app), Some(entity_manager)) = (
            self.entity_modifiers.get(&TypeId::of::<T>()),
            self.app.as_ref(),
            self.entity_manager.upgrade(),
        ) {
            if let Some(app) = weak_app.upgrade() {
                for modifier in modifiers {
                    let casted = modifier.downcast_ref::<EventFunction<T>>().unwrap();
                    (casted.f)(
                        &event,
                        app.borrow_mut(),
                        entity_manager.borrow_mut().deref_mut(),
                    );
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
                delta: MouseScrollDelta::LineDelta(hori, vert),
                phase,
            } => {
                self.trigger(MouseScroll {
                    vertical_lines: vert,
                    horizontal_lines: hori,
                    phase,
                });
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
            DeviceEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(hori, vert),
            } => {
                self.trigger(RawMouseScroll {
                    vertical_delta: vert,
                    horizontal_delta: hori,
                });
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

/// holds the function pointer to the entity system event function
struct EventFunction<T: Any> {
    pub(crate) f: fn(&T, RefMut<Box<dyn FallingLeafApp>>, &mut EntityManager),
}

pub mod events {
    use crate::engine::EngineMode;
    use crate::glm;
    use crate::systems::audio_system::VolumeKind;
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

    /// event that gets triggered when the the sound engines' volume is supposed to change (e.g. when an UI slider is used)
    #[derive(Debug, Clone, PartialEq)]
    pub struct AudioVolumeChanged {
        pub kind: VolumeKind,
        pub new_volume: f32,
    }

    /// global change of the engine mode
    #[derive(Debug, Clone, PartialEq)]
    pub struct EngineModeChange {
        pub new_mode: EngineMode,
    }

    /// change of the users camera position used for rendering and audio processing
    #[derive(Debug, Clone, PartialEq)]
    pub struct CamPositionChange {
        pub new_pos: glm::Vec3,
        pub new_focus: glm::Vec3,
    }

    /// toggles the fps cap functionality
    #[derive(Debug, Clone, PartialEq)]
    pub struct FPSCapToggle;

    /// changes the engines fps cap that is used if fps capping is enabled
    #[derive(Debug, Clone, PartialEq)]
    pub struct FPSCapChanged {
        pub new_fps: f64,
    }

    /// changes the animation speed of the rendering system
    #[derive(Debug, Clone, PartialEq)]
    pub struct AnimationSpeedChange {
        pub new_animation_speed: f32,
    }
}
