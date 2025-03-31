use crate::internal_prelude::*;
use glutin::config::{Config, ConfigTemplateBuilder};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, GlProfile, NotCurrentContext, PossiblyCurrentContext,
    Version,
};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use std::ffi::{CStr, CString};
use std::num::NonZeroU32;
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorGrabMode, CursorIcon, Fullscreen, Window};

/// The system that is responsible for managing the window and its backend attributes.
pub struct VideoSystem {
    pub(crate) config_template: ConfigTemplateBuilder,
    pub(crate) display_builder: DisplayBuilder,
    pub(crate) not_current_gl_context: Option<NotCurrentContext>,
    pub(crate) gl_context: Option<PossiblyCurrentContext>,
    pub(crate) gl_surface: Option<Surface<WindowSurface>>,
    pub(crate) window: Option<Window>,
    current_fps: f64,
    last_draw_time: Instant,
    bg_fps_cap: Option<f64>,
    fps_cap: Option<f64>,
    stored_config: EngineAttributes,
    skipped_first_resize: bool,
    came_out_of_fs: Cell<bool>,
    pub(crate) mouse_cam_sens: Option<f32>,
    forced_viewport_ratio: Option<f32>,
}

impl VideoSystem {
    /// creates a new video state
    pub(crate) fn new(config: EngineAttributes) -> Self {
        let window_attributes = config.generate_win_attrs();

        let config_template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_multisampling(4)
            .with_stencil_size(8)
            .with_transparency(window_attributes.transparent);

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

        Self {
            config_template,
            display_builder,
            not_current_gl_context: None,
            gl_context: None,
            gl_surface: None,
            window: None,
            current_fps: 0f64,
            last_draw_time: Instant::now(),
            bg_fps_cap: config.bg_fps_cap,
            fps_cap: config.fps_cap,
            stored_config: config,
            skipped_first_resize: false,
            came_out_of_fs: Cell::new(false),
            mouse_cam_sens: None,
            forced_viewport_ratio: None,
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

        log::info!(
            "Picked a rendering backend config with {} samples.",
            gl_config.num_samples()
        );

        let raw_window_handle = window
            .as_ref()
            .and_then(|window| window.window_handle().ok())
            .map(|handle| handle.as_raw());

        let gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new()
            .with_profile(GlProfile::Core)
            .with_context_api(ContextApi::OpenGl(None))
            .build(raw_window_handle);

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
                eprintln!("Error setting VSYNC: {res:?}");
            }
        } else if let Err(res) = self.disable_vsync() {
            eprintln!("Error setting VSYNC: {res:?}");
        }

        Ok(())
    }

    /// called when the engine application is suspended
    pub(crate) fn on_suspended(&mut self) {
        // this event is only raised on Android, where the backing NativeWindow for a GL Surface can appear and disappear at any moment
        log::info!("Android window removed.");

        // destroy the GL Surface and un-current the GL Context before ndk-glue releases the window back to the system
        let gl_context = self.gl_context.take().unwrap();
        self.gl_surface = None;
        self.window = None;

        assert!(self
            .not_current_gl_context
            .replace(gl_context.make_not_current().unwrap())
            .is_none());
    }

    /// Enables VSYNC for the rendering backend.
    pub fn enable_vsync(&mut self) -> Result<(), String> {
        log::debug!("Enabled VSYNC.");
        if let (Some(gl_surface), Some(gl_context)) =
            (self.gl_surface.as_ref(), self.gl_context.as_ref())
        {
            return gl_surface
                .set_swap_interval(gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
                .map_err(|err| err.to_string());
        }
        Err(String::from("VSYNC enable failed."))
    }

    /// Disables VSYNC for the rendering backend.
    pub fn disable_vsync(&mut self) -> Result<(), String> {
        log::debug!("Disabled VSYNC.");
        if let (Some(gl_surface), Some(gl_context)) =
            (self.gl_surface.as_ref(), self.gl_context.as_ref())
        {
            return gl_surface
                .set_swap_interval(gl_context, SwapInterval::DontWait)
                .map_err(|err| err.to_string());
        }
        Err(String::from("VSYNC disable failed."))
    }

    /// Call the opengl window swap.
    pub(crate) fn swap_window(&self) {
        if let (Some(gl_surface), Some(gl_context)) =
            (self.gl_surface.as_ref(), self.gl_context.as_ref())
        {
            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }

    /// requests a redraw of the winit window
    pub(crate) fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    /// resets the internal timer for the engine update loop
    pub(crate) fn update_draw_timer(&mut self) {
        let elapsed_draw_time = self.last_draw_time.elapsed();
        self.last_draw_time = Instant::now();
        self.current_fps = 1.0 / elapsed_draw_time.as_secs_f64();
    }

    /// checks wether or not a full engine update loop should occur
    pub(crate) fn should_redraw(&self) -> bool {
        let elapsed = self.last_draw_time.elapsed();
        let user_cap = self
            .fps_cap
            .is_none_or(|fps| elapsed >= Duration::from_secs_f64(1.0 / fps));

        self.bg_fps_cap.map_or(user_cap, |fps| {
            if self.window.as_ref().is_some_and(|win| !win.has_focus()) {
                elapsed >= Duration::from_secs_f64(1.0 / fps)
            } else {
                user_cap
            }
        })
    }

    /// returns the current rendering viewport ratio
    pub(crate) fn current_viewport_ratio(&self) -> f32 {
        if let Some(ratio) = self.forced_viewport_ratio {
            ratio
        } else {
            let win_size = self.window_resolution();
            win_size.width as f32 / win_size.height as f32
        }
    }

    /// event listening function for window resizes
    pub(crate) fn on_window_resize(&mut self, event: &WindowResize) {
        // Some platforms like EGL require resizing GL surface to update the size.
        // Notable platforms here are Wayland and macOS, others don't require it.
        // It's wise to resize it for portability reasons.
        if let (Some(gl_surface), Some(gl_context), Some(window)) = (
            self.gl_surface.as_ref(),
            self.gl_context.as_ref(),
            self.window.as_ref(),
        ) {
            if event.width == 0 || event.height == 0 {
                return;
            }

            let mut size_to_use = (event.width, event.height);
            if self.skipped_first_resize {
                if window.fullscreen().is_none() && !self.came_out_of_fs.get() {
                    if let Some(enforced_ratio) = self.stored_config.enforced_ratio {
                        // enforce window side ratio
                        let corrected_height = (event.width as f32 / enforced_ratio) as u32;
                        if corrected_height != event.height {
                            if let Some(rs) = window.request_inner_size(PhysicalSize::new(
                                event.width,
                                corrected_height,
                            )) {
                                size_to_use = (rs.width, rs.height);
                            }
                        }
                    }
                } else {
                    self.came_out_of_fs.set(false);
                }
            } else {
                self.skipped_first_resize = true;
            }

            gl_surface.resize(
                gl_context,
                NonZeroU32::new(size_to_use.0).unwrap(),
                NonZeroU32::new(size_to_use.1).unwrap(),
            );

            let mut vp_start = (0, 0);
            if let Some(ratio) = self.forced_viewport_ratio {
                let regular_ratio = size_to_use.0 as f32 / size_to_use.1 as f32;
                if ratio < regular_ratio {
                    // viewport is less wide
                    let forced_size = ((size_to_use.1 as f32 * ratio) as u32, size_to_use.1);
                    vp_start.0 = (size_to_use.0 - forced_size.0) / 2;
                    size_to_use = forced_size;
                } else if ratio > regular_ratio {
                    // viewport is more wide
                    let forced_size = (size_to_use.0, (size_to_use.0 as f32 / ratio) as u32);
                    vp_start.1 = (size_to_use.1 - forced_size.1) / 2;
                    size_to_use = forced_size;
                }
            }
            unsafe {
                #[rustfmt::skip]
                gl::Viewport(vp_start.0 as GLint, vp_start.1 as GLint, size_to_use.0 as GLsizei, size_to_use.1 as GLsizei);
                #[rustfmt::skip]
                gl::Scissor(vp_start.0 as GLint, vp_start.1 as GLint, size_to_use.0 as GLsizei, size_to_use.1 as GLsizei);
            }
        }
    }

    /// Gets the current FPS.
    #[inline]
    pub fn current_fps(&self) -> f64 {
        self.current_fps
    }

    /// Gets the current inner window resolution in physical size pixels (width, height).
    pub fn window_resolution(&self) -> PhysicalSize<u32> {
        self.window.as_ref().unwrap().inner_size()
    }

    /// Gets the current DPI scale factor of the window.
    pub fn dpi_scale(&self) -> f64 {
        self.window.as_ref().unwrap().scale_factor()
    }

    /// Sets the optional FPS cap value for the rendering process. This is the same setting that you could also specify in the ``EngineAttributes`` at start-up.
    pub fn set_fps_cap(&mut self, new_cap: Option<f64>) {
        log::trace!("Set FPS cap: {new_cap:?}.");
        self.fps_cap = new_cap;
    }

    /// Sets the optional FPS cap value for the rendering process for when the app is out of focus. This is the same setting that you could also specify in the ``EngineAttributes`` at start-up.
    pub fn set_bg_fps_cap(&mut self, new_cap: Option<f64>) {
        log::trace!("Set background FPS cap: {new_cap:?}.");
        self.bg_fps_cap = new_cap;
    }

    /// Changes the title bar text in the window.
    pub fn set_window_title(&self, title: impl AsRef<str>) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(title.as_ref());
        }
    }

    /// Changes the appearance of the windows' cursor.
    pub fn set_cursor(&self, cursor: CursorIcon) {
        if let Some(window) = self.window.as_ref() {
            window.set_cursor(cursor);
        }
    }

    /// Brings the window into focus.
    pub fn focus_window(&self) {
        if let Some(window) = self.window.as_ref() {
            window.focus_window();
            log::trace!("Focussed Window.");
        }
    }

    /// Enables/disables the grab mode for the cursor (makes it unable to leave the window).
    pub fn set_cursor_confined(&self, flag: bool) {
        if let Some(window) = self.window.as_ref() {
            if flag {
                window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
            } else {
                window.set_cursor_grab(CursorGrabMode::None).unwrap();
            }
            log::trace!("Set cursor confined: {flag:?}.");
        }
    }

    /// Enables/disables fullscreen for the window.
    pub fn set_fullscreen(&self, flag: bool) {
        if let Some(window) = self.window.as_ref() {
            if flag {
                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
            } else {
                if window.fullscreen().is_some() {
                    self.came_out_of_fs.set(true);
                }
                window.set_fullscreen(None);
            }
            log::debug!("Set fullscreen: {flag:?}.");
        }
    }

    /// Makes the cursor visible/invisible.
    pub fn set_cursor_visible(&self, flag: bool) {
        if let Some(window) = self.window.as_ref() {
            window.set_cursor_visible(flag);
            log::trace!("Set cursor visibility: {flag:?}.");
        }
    }

    /// Enables/disables the link to the first person 3D camera control for the mouse with some senstivity (default is ``None``).
    pub fn set_mouse_fpp_cam_control(&mut self, sensitivity: Option<f32>) {
        match sensitivity {
            None => {
                self.set_cursor_visible(true);
                self.set_cursor_confined(false);
            }
            Some(_) => {
                self.set_cursor_visible(false);
                self.set_cursor_confined(true);
            }
        }
        self.mouse_cam_sens = sensitivity;
        log::trace!("Set mouse cam control sensitivity: {sensitivity:?}.");
    }

    /// Forces the rendering viewport to a ratio (width / height) regardless of the window size. Probably only useful without transparency enabled.
    pub fn force_viewport_ratio(&mut self, ratio: Option<f32>) {
        self.forced_viewport_ratio = ratio;
        log::debug!("Forcing viewport ratio: {ratio:?}.");
    }
}

/// prints info about the used gl renderer
fn log_gl_config() {
    if let Some(renderer) = get_gl_string(gl::RENDERER) {
        log::info!("Running on {}.", renderer.to_string_lossy());
    }
    if let Some(version) = get_gl_string(gl::VERSION) {
        log::info!("OpenGL Version: {}.", version.to_string_lossy());
    }
    if let Some(shaders_version) = get_gl_string(gl::SHADING_LANGUAGE_VERSION) {
        log::info!("Shaders version: {}.", shaders_version.to_string_lossy());
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
