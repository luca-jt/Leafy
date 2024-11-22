use crate::ecs::component::TouchTime;
use crate::ecs::entity_manager::EntityManager;
use crate::engine_builder::EngineAttributes;
use crate::systems::animation_system::{move_cam, stop_cam, AnimationSystem};
use crate::systems::audio_system::AudioSystem;
use crate::systems::event_system::events::*;
use crate::systems::event_system::EventSystem;
use crate::systems::rendering_system::RenderingSystem;
use crate::systems::video_system::{mouse_move_cam, VideoSystem};
use crate::utils::tools::{shared_ptr, SharedPtr};
use std::any::Any;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::error::Error;
use std::fmt::Debug;
use std::ops::Deref;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowId;

/// main engine
pub struct Engine<A: FallingLeafApp> {
    app: Option<SharedPtr<A>>,
    exit_state: Option<Result<(), Box<dyn Error>>>,
    should_quit: Cell<bool>,
    rendering_system: Option<SharedPtr<RenderingSystem>>,
    audio_system: SharedPtr<AudioSystem>,
    event_system: RefCell<EventSystem<A>>,
    animation_system: SharedPtr<AnimationSystem>,
    entity_manager: SharedPtr<EntityManager>,
    video_system: SharedPtr<VideoSystem>,
}

impl<A: FallingLeafApp> Engine<A> {
    /// engine setup on startup
    pub(crate) fn new(config: EngineAttributes) -> Self {
        let video_system = shared_ptr(VideoSystem::new(config));
        let audio_system = shared_ptr(AudioSystem::new());
        let animation_system = shared_ptr(AnimationSystem::new());
        let entity_manager = shared_ptr(EntityManager::new());
        let mut event_system = EventSystem::new();

        event_system.add_listener::<WindowResize>(&video_system);
        event_system.add_listener::<CamPositionChange>(&audio_system);
        event_system.add_listener::<AnimationSpeedChange>(&audio_system);
        event_system.add_listener::<EngineModeChange>(&audio_system);
        event_system.add_listener::<AnimationSpeedChange>(&animation_system);
        event_system.add_listener::<EngineModeChange>(&animation_system);
        event_system.add_modifier(mouse_move_cam);
        event_system.add_modifier(move_cam);
        event_system.add_modifier(stop_cam);

        Self {
            app: None,
            exit_state: Some(Ok(())),
            should_quit: Cell::new(false),
            rendering_system: None,
            audio_system,
            event_system: RefCell::new(event_system),
            animation_system,
            entity_manager,
            video_system,
        }
    }

    /// runs the main loop
    pub fn run(&mut self, app: A) -> Result<(), Box<dyn Error>> {
        self.app = Some(shared_ptr(app));
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self)?;
        self.exit_state.take().unwrap()
    }

    /// gets called every frame and contains the main app logic
    fn on_frame_redraw(&mut self) {
        self.app_mut().on_frame_update(self);

        self.audio_system_mut()
            .update(self.entity_manager().deref());

        self.animation_system_mut().update(self);

        self.rendering_system_mut()
            .update_light_sources(self.entity_manager().deref());

        self.rendering_system_mut()
            .render(self.entity_manager().deref());
    }

    /// access to the stored app
    pub fn app(&self) -> Ref<A> {
        self.app.as_ref().unwrap().borrow()
    }

    /// mutable access to the stored app
    pub fn app_mut(&self) -> RefMut<A> {
        self.app.as_ref().unwrap().borrow_mut()
    }

    /// access to the engines animation system
    pub fn animation_system(&self) -> Ref<AnimationSystem> {
        self.animation_system.borrow()
    }

    /// mutable access to the engines animation system
    pub fn animation_system_mut(&self) -> RefMut<AnimationSystem> {
        self.animation_system.borrow_mut()
    }

    /// access to the engines rendering system
    pub fn rendering_system(&self) -> Ref<RenderingSystem> {
        self.rendering_system.as_ref().unwrap().borrow()
    }

    /// mutable access to the engines rendering system
    pub fn rendering_system_mut(&self) -> RefMut<RenderingSystem> {
        self.rendering_system.as_ref().unwrap().borrow_mut()
    }

    /// access to the engines audio system
    pub fn audio_system(&self) -> Ref<AudioSystem> {
        self.audio_system.borrow()
    }

    /// mutable access to the engines audio system
    pub fn audio_system_mut(&self) -> RefMut<AudioSystem> {
        self.audio_system.borrow_mut()
    }

    /// access to the engines video system
    pub fn video_system(&self) -> Ref<VideoSystem> {
        self.video_system.borrow()
    }

    /// mutable access to the engines video system
    pub fn video_system_mut(&self) -> RefMut<VideoSystem> {
        self.video_system.borrow_mut()
    }

    /// access to the engines event system
    pub fn event_system(&self) -> Ref<EventSystem<A>> {
        self.event_system.borrow()
    }

    /// mutable access to the engines event system
    pub fn event_system_mut(&self) -> RefMut<EventSystem<A>> {
        self.event_system.borrow_mut()
    }

    /// access to the engines entity manager
    pub fn entity_manager(&self) -> Ref<EntityManager> {
        self.entity_manager.borrow()
    }

    /// mutable access to the engines entity manager
    pub fn entity_manager_mut(&self) -> RefMut<EntityManager> {
        self.entity_manager.borrow_mut()
    }

    /// quits the running engine and exit the event loop
    pub fn quit(&self) {
        self.should_quit.set(true);
    }

    /// triggers an engine-wide event in the event system and call all relevant functions/listeners
    pub fn trigger_event<T: Any + Debug>(&self, event: T) {
        self.event_system().trigger(event, self);
    }
}

impl<A: FallingLeafApp> ApplicationHandler for Engine<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.exit_state = Some(self.video_system.borrow_mut().on_resumed(event_loop));

        self.rendering_system = Some(shared_ptr(RenderingSystem::new()));
        self.event_system_mut()
            .add_listener::<CamPositionChange>(self.rendering_system.as_ref().unwrap());
        self.event_system_mut()
            .add_listener::<WindowResize>(self.rendering_system.as_ref().unwrap());

        self.app_mut().init(self);
        self.animation_system_mut().time_of_last_sim = TouchTime::now();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if self.video_system().should_redraw() {
                    self.video_system_mut().update_draw_timer();
                    self.on_frame_redraw();
                    self.video_system().swap_window();
                }
                if self.should_quit.get() {
                    event_loop.exit();
                }
                self.video_system().request_redraw();
            }
            _ => self.event_system().parse_winit_window_event(event, self),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.event_system()
            .parse_winit_device_event(device_id, event, self);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.video_system_mut().on_suspended();
    }
}

/// all necessary app functionality to run the engine with
pub trait FallingLeafApp: Sized + 'static {
    /// initialize the app (e.g. add event handling)
    fn init(&mut self, engine: &Engine<Self>);
    /// run this update code every frame
    fn on_frame_update(&mut self, engine: &Engine<Self>);
}

/// all possible states of the engine that influence its behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineMode {
    Running,
    Paused,
    Editor,
}
