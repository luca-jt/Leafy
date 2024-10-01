use crate::engine_builder::EngineAttributes;
use crate::systems::event_system::events::{KeyPress, WindowResize};
use crate::systems::event_system::EventObserver;
use gl::types::GLsizei;
use glutin::config::{Config, ConfigTemplateBuilder};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentContext, PossiblyCurrentContext, Version,
};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::num::NonZeroU32;
use std::time::{Duration, Instant};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, CursorIcon, Fullscreen, Window};

/// holds the video backend attributes
pub struct VideoSystem {
    pub(crate) config_template: ConfigTemplateBuilder,
    pub(crate) display_builder: DisplayBuilder,
    pub(crate) not_current_gl_context: Option<NotCurrentContext>,
    pub(crate) gl_context: Option<PossiblyCurrentContext>,
    pub(crate) gl_surface: Option<Surface<WindowSurface>>,
    pub(crate) window: Option<Window>,
    current_fps: f64,
    frame_start_time: Instant,
    fps_cap: Option<f64>,
    stored_config: EngineAttributes,
}

impl VideoSystem {
    /// creates a new video state
    pub(crate) fn new(config: EngineAttributes) -> Self {
        let window_attributes = config.generate_win_attrs();

        let config_template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_transparency(cfg!(cgl_backend));

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

        Self {
            config_template,
            display_builder,
            not_current_gl_context: None,
            gl_context: None,
            gl_surface: None,
            window: None,
            current_fps: 0f64,
            frame_start_time: Instant::now(),
            fps_cap: config.fps_cap,
            stored_config: config,
        }
    }

    /// called when the engine application is resumed
    pub(crate) fn on_resumed(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), Box<dyn Error>> {
        let (mut window, gl_config) = match self.display_builder.clone().build(
            event_loop,
            self.config_template.clone(),
            gl_config_picker,
        ) {
            Ok(ok) => ok,
            Err(e) => {
                event_loop.exit();
                return Err(e);
            }
        };

        println!("Picked a config with {} samples", gl_config.num_samples());

        let raw_window_handle = window
            .as_ref()
            .and_then(|window| window.window_handle().ok())
            .map(|handle| handle.as_raw());

        let gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(raw_window_handle);

        let legacy_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
            .build(raw_window_handle);

        let not_current_gl_context = self
            .not_current_gl_context
            .take()
            .unwrap_or_else(|| unsafe {
                gl_display
                    .create_context(&gl_config, &context_attributes)
                    .unwrap_or_else(|_| {
                        gl_display
                            .create_context(&gl_config, &fallback_context_attributes)
                            .unwrap_or_else(|_| {
                                gl_display
                                    .create_context(&gl_config, &legacy_context_attributes)
                                    .expect("failed to create context")
                            })
                    })
            });

        let window = window.take().unwrap_or_else(|| {
            glutin_winit::finalize_window(
                event_loop,
                self.stored_config.generate_win_attrs(),
                &gl_config,
            )
            .unwrap()
        });

        let attrs = window
            .build_surface_attributes(Default::default())
            .expect("Failed to build surface attributes");

        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();
        // The context needs to be current for the Renderer to set up shaders and
        // buffers. It also performs function loading, which needs a current context on WGL.

        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });
        log_gl_config();

        // refresh the video state
        assert!(
            self.gl_context.replace(gl_context).is_none()
                && self.gl_surface.replace(gl_surface).is_none()
                && self.window.replace(window).is_none()
        );

        if self.stored_config.use_vsync {
            if let Err(res) = self.enable_vsync() {
                eprintln!("Error setting vsync: {res:?}");
            }
        } else if let Err(res) = self.disable_vsync() {
            eprintln!("Error setting vsync: {res:?}");
        }

        Ok(())
    }

    /// called when the engine application is suspended
    pub(crate) fn on_suspended(&mut self) {
        // this event is only raised on Android, where the backing NativeWindow for a GL Surface can appear and disappear at any moment
        println!("Android window removed");

        // destroy the GL Surface and un-current the GL Context before ndk-glue releases the window back to the system
        let gl_context = self.gl_context.take().unwrap();
        self.gl_surface = None;
        self.window = None;

        assert!(self
            .not_current_gl_context
            .replace(gl_context.make_not_current().unwrap())
            .is_none());
    }

    /// enables vsync for opengl
    pub fn enable_vsync(&mut self) -> Result<(), String> {
        if let (Some(gl_surface), Some(gl_context)) =
            (self.gl_surface.as_ref(), self.gl_context.as_ref())
        {
            return gl_surface
                .set_swap_interval(gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
                .map_err(|err| err.to_string());
        }
        Err(String::from("vsync disable failed"))
    }

    /// disables vsync for opengl
    pub fn disable_vsync(&mut self) -> Result<(), String> {
        if let (Some(gl_surface), Some(gl_context)) =
            (self.gl_surface.as_ref(), self.gl_context.as_ref())
        {
            return gl_surface
                .set_swap_interval(gl_context, SwapInterval::DontWait)
                .map_err(|err| err.to_string());
        }
        Err(String::from("vsync disable failed"))
    }

    /// call the opengl window swap
    pub(crate) fn swap_window(&self) {
        if let (Some(window), Some(gl_surface), Some(gl_context)) = (
            self.window.as_ref(),
            self.gl_surface.as_ref(),
            self.gl_context.as_ref(),
        ) {
            window.request_redraw();
            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }

    /// caps the fps of the event loop if the setting requires it
    pub(crate) fn try_cap_fps(&mut self) {
        if let Some(fps_cap) = self.fps_cap {
            let elapsed_frame_time = self.frame_start_time.elapsed();
            let max_frame_time = Duration::from_secs_f64(1.0 / fps_cap);
            if elapsed_frame_time < max_frame_time {
                std::thread::sleep(max_frame_time - elapsed_frame_time);
            }
            self.current_fps = (1.0 / self.frame_start_time.elapsed().as_secs_f64()).round();
            self.frame_start_time = Instant::now();
        }
    }

    /// gets the current fps in seconds
    pub fn current_fps(&self) -> f64 {
        self.current_fps
    }

    /// set the optional fps cap value for the rendering process
    pub fn set_fps_cap(&mut self, new_cap: Option<f64>) {
        self.fps_cap = new_cap;
    }

    /// changes the title bar text in the window
    pub fn set_window_title(&self, title: &str) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(title);
        }
    }

    /// changes the appearance of the windows' cursor
    pub fn set_cursor(&self, cursor: CursorIcon) {
        if let Some(window) = self.window.as_ref() {
            window.set_cursor(cursor);
        }
    }

    /// brings the window into focus
    pub fn focus_window(&self) {
        if let Some(window) = self.window.as_ref() {
            window.focus_window();
        }
    }

    /// enables/disables the grab mode for the cursor (makes it unable to leave the window)
    pub fn set_cursor_grab_mode(&self, flag: bool) {
        if let Some(window) = self.window.as_ref() {
            if flag {
                window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
            } else {
                window.set_cursor_grab(CursorGrabMode::None).unwrap();
            }
        }
    }

    /// enables/disables fullscreen for the window
    pub fn set_fullscreen(&self, flag: bool) {
        if let Some(window) = self.window.as_ref() {
            if flag {
                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
            } else {
                window.set_fullscreen(None);
            }
        }
    }

    /// makes the cursor visible/invisible
    pub fn set_cursor_visible(&self, flag: bool) {
        if let Some(window) = self.window.as_ref() {
            window.set_cursor_visible(flag);
        }
    }
}

impl EventObserver<KeyPress> for VideoSystem {
    fn on_event(&mut self, event: &KeyPress) {
        if event.key == KeyCode::F11 {
            // toggle fullscreen
            if let Some(window) = self.window.as_ref() {
                if window.fullscreen().is_some() {
                    window.set_fullscreen(None);
                } else {
                    window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                }
            }
        }
    }
}

impl EventObserver<WindowResize> for VideoSystem {
    fn on_event(&mut self, event: &WindowResize) {
        // Some platforms like EGL require resizing GL surface to update the size.
        // Notable platforms here are Wayland and macOS, others don't require it.
        // It's wise to resize it for portability reasons.
        if let (Some(gl_surface), Some(gl_context), Some(_window)) = (
            self.gl_surface.as_ref(),
            self.gl_context.as_ref(),
            self.window.as_ref(),
        ) {
            if event.width == 0 || event.height == 0 {
                return;
            }
            /*let corrected_height = (event.width as f32 * INV_WIN_RATIO) as u32;
            let final_size: (u32, u32);

            if corrected_height != event.height {
                let returned_size =
                    window.request_inner_size(PhysicalSize::new(event.width, corrected_height));
                if let Some(rs) = returned_size {
                    final_size = (rs.width, rs.height);
                } else {
                    return;
                }
            } else {
                final_size = (event.width, event.height);
            }*/
            gl_surface.resize(
                gl_context,
                NonZeroU32::new(event.width).unwrap(),
                NonZeroU32::new(event.height).unwrap(),
            );
            unsafe {
                gl::Viewport(0, 0, event.width as GLsizei, event.height as GLsizei);
                gl::Scissor(0, 0, event.width as GLsizei, event.height as GLsizei);
            }
        }
    }
}

/// prints info about the used gl renderer
fn log_gl_config() {
    if let Some(renderer) = get_gl_string(gl::RENDERER) {
        println!("Running on {}", renderer.to_string_lossy());
    }
    if let Some(version) = get_gl_string(gl::VERSION) {
        println!("OpenGL Version {}", version.to_string_lossy());
    }
    if let Some(shaders_version) = get_gl_string(gl::SHADING_LANGUAGE_VERSION) {
        println!("Shaders version on {}", shaders_version.to_string_lossy());
    }
}

/// find the config with the maximum number of samples
fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false)
                & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}

/// retrieves a string value from gl
fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}
