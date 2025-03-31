use crate::ecs::entity_manager::EntityManager;
use crate::internal_prelude::*;
use crate::systems::animation_system::AnimationSystem;
use crate::systems::audio_system::AudioSystem;
use crate::systems::event_system::{Event, EventSystem};
use crate::systems::general::*;
use crate::systems::rendering_system::RenderingSystem;
use crate::systems::video_system::VideoSystem;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowId;

/// The main engine. This is the basis for all apps that use this library. You can control all functionalities of it with the methods of this struct. All of the different systems have their respective accessor functions in both immutable and mutable variants. This way you can control the interior mutability of the engine.
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
    /// Engine setup on startup.
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

    /// Runs the main loop of the engine and is the main function that is called after the creation of the engine. This takes in your app struct that you created that implements the ``FallingLeafApp`` trait.
    pub fn run(&mut self, app: A) -> Result<(), Box<dyn Error>> {
        self.app = Some(RefCell::new(app));
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self)?;
        self.exit_state.take().unwrap()
    }

    /// Gets called every frame and contains the main engine logic.
    fn on_frame_redraw(&mut self) {
        self.app_mut().on_frame_update(self);

        self.time_step_sim();

        self.audio_system_mut()
            .update(self.entity_manager_mut().deref_mut());

        self.rendering_system_mut()
            .render(self.entity_manager().deref());
    }

    /// All of the time-sensitive simulations for a single time step.
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

    /// Access to the stored app. This way you can access your app struct in event functions. You should not use this inside the ``FallingLeafApp`` trait functions as that would harm the dynamically checked borrowing rules.
    pub fn app(&self) -> Ref<A> {
        self.app.as_ref().unwrap().borrow()
    }

    /// Mutable access to the stored app. This way you can access your app struct in event functions. You should not use this inside the ``FallingLeafApp`` trait functions as that would harm the dynamically checked borrowing rules.
    pub fn app_mut(&self) -> RefMut<A> {
        self.app.as_ref().unwrap().borrow_mut()
    }

    /// Access to the engine's animation system.
    pub fn animation_system(&self) -> Ref<AnimationSystem> {
        self.animation_system.borrow()
    }

    /// Autable access to the engine's animation system.
    pub fn animation_system_mut(&self) -> RefMut<AnimationSystem> {
        self.animation_system.borrow_mut()
    }

    /// Access to the engine's rendering system.
    pub fn rendering_system(&self) -> Ref<RenderingSystem> {
        self.rendering_system.as_ref().unwrap().borrow()
    }

    /// Mutable access to the engine's rendering system.
    pub fn rendering_system_mut(&self) -> RefMut<RenderingSystem> {
        self.rendering_system.as_ref().unwrap().borrow_mut()
    }

    /// Access to the engine's audio system.
    pub fn audio_system(&self) -> Ref<AudioSystem> {
        self.audio_system.borrow()
    }

    /// Mutable access to the engine's audio system.
    pub fn audio_system_mut(&self) -> RefMut<AudioSystem> {
        self.audio_system.borrow_mut()
    }

    /// Access to the engine's video system.
    pub fn video_system(&self) -> Ref<VideoSystem> {
        self.video_system.borrow()
    }

    /// Mutable access to the engine's video system.
    pub fn video_system_mut(&self) -> RefMut<VideoSystem> {
        self.video_system.borrow_mut()
    }

    /// Access to the engine's event system.
    pub fn event_system(&self) -> Ref<EventSystem<A>> {
        self.event_system.borrow()
    }

    /// Mutable access to the engine's event system.
    pub fn event_system_mut(&self) -> RefMut<EventSystem<A>> {
        self.event_system.borrow_mut()
    }

    /// Access to the engine's entity manager.
    pub fn entity_manager(&self) -> Ref<EntityManager> {
        self.entity_manager.borrow()
    }

    /// Mutable access to the engine's entity manager.
    pub fn entity_manager_mut(&self) -> RefMut<EntityManager> {
        self.entity_manager.borrow_mut()
    }

    /// The current mode of the engine.
    pub fn mode(&self) -> EngineMode {
        self.mode.get()
    }

    /// Quits the running engine and exit the event loop.
    pub fn quit(&self) {
        self.should_quit.set(true);
    }

    /// Triggers an engine-wide event in the event system and call all relevant functions/listeners.
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

/// All necessary app functionality to run the engine with. An app struct that is used with the engine has to implement this trait.
pub trait FallingLeafApp: Sized + 'static {
    /// Initializes the app (e.g. add event handling, loading data, settings) at engine start-up. This function will only run once.
    fn init(&mut self, engine: &Engine<Self>);
    /// Runs every frame and is supposed to be used to implement the logic of your app struct.
    fn on_frame_update(&mut self, engine: &Engine<Self>);
}

/// All possible states of the engine that influence its behavior. Can be changed by triggering an ``EngineModeChange`` user space event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineMode {
    Running,
    Editor,
}
