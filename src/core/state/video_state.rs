use crate::systems::event_system::{EventObserver, FLEventData};
use crate::utils::constants::{INV_WIN_RATIO, MIN_WIN_HEIGHT, MIN_WIN_WIDTH, WIN_TITLE};
use crate::utils::file::get_image_path;
use sdl2::image::LoadSurface;
use sdl2::keyboard::Keycode;
use sdl2::surface::Surface;
use sdl2::video::{FullscreenType, GLProfile, SwapInterval};

/// holds the video backend attributes
pub struct VideoState {
    pub sdl_context: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    pub window: sdl2::video::Window,
    _gl_ctx: sdl2::video::GLContext,
}

impl VideoState {
    /// creates a new video state
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_double_buffer(true);
        gl_attr.set_multisample_samples(4);
        gl_attr.set_framebuffer_srgb_compatible(true);
        gl_attr.set_context_version(4, 5);

        let mut window = video_subsystem
            .window(WIN_TITLE, MIN_WIN_WIDTH, MIN_WIN_HEIGHT)
            .opengl()
            .position_centered()
            .allow_highdpi()
            .resizable()
            .build()
            .unwrap();

        window.set_icon(Surface::from_file(get_image_path("icon.png")).unwrap());
        window
            .set_minimum_size(MIN_WIN_WIDTH, MIN_WIN_HEIGHT)
            .unwrap();

        let _gl_ctx = window.gl_create_context().unwrap();
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);
        video_subsystem
            .gl_set_swap_interval(SwapInterval::Immediate)
            .unwrap();

        Self {
            sdl_context,
            video_subsystem,
            window,
            _gl_ctx,
        }
    }

    /// enables vsync for opengl
    pub fn enable_vsync(&mut self) {
        self.video_subsystem
            .gl_set_swap_interval(SwapInterval::VSync)
            .unwrap();
    }

    /// disables vsync for opengl
    pub fn disable_vsync(&mut self) {
        self.video_subsystem
            .gl_set_swap_interval(SwapInterval::Immediate)
            .unwrap();
    }

    /// call the opengl window swap
    pub fn swap_window(&self) {
        self.window.gl_swap_window();
    }
}

impl EventObserver for VideoState {
    fn on_event(&mut self, event: &FLEventData) {
        if let FLEventData::KeyPress(key) = event {
            if *key == Keycode::F11 {
                // toggle fullscreen
                match self.window.fullscreen_state() {
                    FullscreenType::Off => {
                        self.window.set_fullscreen(FullscreenType::Desktop).unwrap();
                    }
                    FullscreenType::Desktop => {
                        self.window.set_fullscreen(FullscreenType::Off).unwrap();
                    }
                    _ => {
                        panic!("wrong fullscreen type detected");
                    }
                }
            }
        } else if let FLEventData::WindowResize(w, _) = event {
            if !self.window.is_maximized()
                && self.window.fullscreen_state() != FullscreenType::Desktop
            {
                self.window
                    .set_size(*w as u32, (*w as f32 * INV_WIN_RATIO) as u32)
                    .unwrap();
            }
        }
    }
}
