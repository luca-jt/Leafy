use crate::ecs::component::utils::{TimeDuration, TimePoint};
use crate::ecs::entity_manager::EntityManager;
use crate::engine_builder::EngineAttributes;
use crate::systems::animation_system::AnimationSystem;
use crate::systems::audio_system::AudioSystem;
use crate::systems::event_system::{Event, EventSystem};
use crate::systems::general::*;
use crate::systems::rendering_system::RenderingSystem;
use crate::systems::video_system::VideoSystem;
use crate::utils::constants::TIME_STEP;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::error::Error;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowId;

/// main engine
pub struct Engine<A: FallingLeafApp> {
    app: Option<RefCell<A>>,
    exit_state: Option<Result<(), Box<dyn Error>>>,
    should_quit: Cell<bool>,
    pub(crate) mode: Cell<EngineMode>,
    rendering_system: Option<RefCell<RenderingSystem>>,
    audio_system: RefCell<AudioSystem>,
    event_system: RefCell<EventSystem<A>>,
    animation_system: RefCell<AnimationSystem>,
    entity_manager: RefCell<EntityManager>,
    video_system: RefCell<VideoSystem>,
    time_accumulated: TimeDuration,
    time_of_last_sim: TimePoint,
}

impl<A: FallingLeafApp> Engine<A> {
    /// engine setup on startup
    pub(crate) fn new(config: EngineAttributes) -> Self {
        let video_system = VideoSystem::new(config);
        let audio_system = AudioSystem::new();
        let animation_system = AnimationSystem::new();
        let entity_manager = EntityManager::new();
        let mut event_system = EventSystem::new();

        event_system.add_modifier(on_window_resize);
        event_system.add_modifier(on_mode_change);
        event_system.add_modifier(mouse_move_cam);
        event_system.add_modifier(move_cam);
        event_system.add_modifier(stop_cam);
        event_system.add_modifier(on_animation_speed_change);
        event_system.add_modifier(on_cam_position_change);

        Self {
            app: None,
            exit_state: Some(Ok(())),
            should_quit: Cell::new(false),
            mode: Cell::new(EngineMode::Running),
            rendering_system: None,
            audio_system: RefCell::new(audio_system),
            event_system: RefCell::new(event_system),
            animation_system: RefCell::new(animation_system),
            entity_manager: RefCell::new(entity_manager),
            video_system: RefCell::new(video_system),
            time_accumulated: TimeDuration(0.0),
            time_of_last_sim: TimePoint::now(),
        }
    }

    /// runs the main loop
    pub fn run(&mut self, app: A) -> Result<(), Box<dyn Error>> {
        self.app = Some(RefCell::new(app));
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self)?;
        self.exit_state.take().unwrap()
    }

    /// gets called every frame and contains the main app logic
    fn on_frame_redraw(&mut self) {
        self.app_mut().on_frame_update(self);

        self.time_step_sim();

        self.audio_system_mut()
            .update(self.entity_manager_mut().deref_mut());

        self.rendering_system_mut()
            .render(self.entity_manager().deref());
    }

    /// all of the time-sensitive simulations
    fn time_step_sim(&mut self) {
        let dt = self.time_of_last_sim.delta_time();
        let transformed_dt = dt * self.animation_system().animation_speed;
        self.time_accumulated += transformed_dt;
        while self.time_accumulated >= TIME_STEP {
            if self.mode() == EngineMode::Running {
                self.animation_system_mut().update(self);
            }
            self.time_accumulated -= TIME_STEP;
        }
        update_cam(self, dt);
        if self.mode() == EngineMode::Running {
            update_doppler_data(self, transformed_dt);
        }
        self.time_of_last_sim.reset();
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

    /// the current mode of the engine
    pub fn mode(&self) -> EngineMode {
        self.mode.get()
    }

    /// quits the running engine and exit the event loop
    pub fn quit(&self) {
        self.should_quit.set(true);
    }

    /// triggers an engine-wide event in the event system and call all relevant functions/listeners
    pub fn trigger_event<T: Event>(&self, event: T) {
        self.event_system().trigger(event, self);
    }
}

impl<A: FallingLeafApp> ApplicationHandler for Engine<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.exit_state = Some(self.video_system.borrow_mut().on_resumed(event_loop));

        let res = self.video_system().window_resolution();
        self.rendering_system = Some(RefCell::new(RenderingSystem::new(res.width, res.height)));

        self.app_mut().init(self);
        self.time_of_last_sim = TimePoint::now();
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
    Editor,
}
