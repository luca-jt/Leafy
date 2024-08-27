use crate::glm;
use std::cell::RefMut;
use std::error::Error;
use std::ops::{Deref, DerefMut};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowId;

use crate::ecs::entity_manager::EntityManager;
use crate::systems::animation_system::AnimationSystem;
use crate::systems::audio_system::AudioSystem;
use crate::systems::event_system::events::{
    AudioVolumeChanged, CamPositionChange, EngineModeChange, KeyPress, WindowResize,
};
use crate::systems::event_system::EventSystem;
use crate::systems::rendering_system::RenderingSystem;
use crate::systems::video_system::VideoSystem;
use crate::utils::tools::{shared_ptr, SharedPtr};

/// main engine
pub struct Engine {
    app: Option<Box<dyn FallingLeafApp>>,
    exit_state: Option<Result<(), Box<dyn Error>>>,
    rendering_system: Option<SharedPtr<RenderingSystem>>,
    pub audio_system: SharedPtr<AudioSystem>,
    pub event_system: EventSystem,
    pub animation_system: AnimationSystem,
    pub video_system: SharedPtr<VideoSystem>,
}

impl Engine {
    /// engine setup on startup
    pub fn new() -> Self {
        let video_system = shared_ptr(VideoSystem::new());
        let audio_system = shared_ptr(AudioSystem::new());
        let animation_system = AnimationSystem::new();
        let mut event_system = EventSystem::new();

        event_system.add_listener::<KeyPress>(&video_system);
        event_system.add_listener::<WindowResize>(&video_system);
        event_system.add_listener::<AudioVolumeChanged>(&audio_system);
        event_system.add_listener::<CamPositionChange>(&audio_system);

        Self {
            app: None,
            exit_state: Some(Ok(())),
            rendering_system: None,
            audio_system,
            event_system,
            animation_system,
            video_system,
        }
    }

    /// runs the main loop
    pub fn run(&mut self, mut app: impl FallingLeafApp + 'static) -> Result<(), Box<dyn Error>> {
        app.init(self);
        self.app = Some(Box::new(app));
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self)?;
        self.exit_state.take().unwrap()
    }

    /// gets called every frame and contains the main app logic
    fn on_frame_redraw(&mut self) {
        self.audio_system
            .borrow_mut()
            .update(self.app.as_mut().unwrap().entity_manager().deref());
        self.animation_system
            .apply_physics(self.app.as_mut().unwrap().entity_manager().deref_mut());

        self.rendering_system
            .as_mut()
            .unwrap()
            .borrow_mut()
            .render(self.app.as_mut().unwrap().entity_manager().deref());

        self.app
            .as_mut()
            .unwrap()
            .on_frame_update(&mut self.event_system, &self.audio_system);

        self.video_system.borrow().swap_window();
        self.video_system.borrow_mut().try_cap_fps();
    }
}

impl ApplicationHandler for Engine {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.exit_state = Some(self.video_system.borrow_mut().on_resumed(event_loop));
        let cam_data = self.app.as_ref().unwrap().cam_start_data();
        self.rendering_system = Some(shared_ptr(RenderingSystem::new(cam_data.0, cam_data.1)));
        self.event_system
            .add_listener::<EngineModeChange>(self.rendering_system.as_ref().unwrap());
        self.event_system
            .add_listener::<CamPositionChange>(self.rendering_system.as_ref().unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.on_frame_redraw(),
            _ => self.event_system.parse_winit_window_event(event),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.event_system.parse_winit_device_event(device_id, event);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.video_system.borrow_mut().on_suspended();
    }
}

/// all necessary app functionality to run the engine with
pub trait FallingLeafApp {
    /// initialize the app (e.g. add event handling)
    fn init(&mut self, engine: &mut Engine);
    /// run this update code every frame
    fn on_frame_update(
        &mut self,
        event_system: &mut EventSystem,
        audio_system: &SharedPtr<AudioSystem>,
    );
    /// allows for access to the entity manager to be used for all engine operations
    fn entity_manager(&mut self) -> RefMut<EntityManager>;
    /// provides the engine with initial camera position and focus
    fn cam_start_data(&self) -> (glm::Vec3, glm::Vec3);
}

/// all possible states of the engine that influence its behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineMode {
    Running,
    Paused,
    Menu,
}
