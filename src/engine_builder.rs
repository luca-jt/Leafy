use crate::engine::{Engine, FallingLeafApp};
use crate::utils::constants::*;
use std::path::PathBuf;
use winit::dpi::LogicalSize;
use winit::window::{Fullscreen, Theme, Window, WindowAttributes};

#[cfg(target_os = "windows")]
use crate::utils::file::get_image_path;
#[cfg(target_os = "windows")]
use winit::platform::windows::IconExtWindows;
#[cfg(target_os = "windows")]
use winit::window::Icon;

/// builder for the Engine that allows for easy modification of intial parameters
pub struct EngineAttributes {
    title: &'static str,
    pub(crate) fps_cap: Option<f64>,
    size: (u32, u32),
    min_size: Option<(u32, u32)>,
    pub(crate) enforced_ratio: Option<f32>,
    transparent: bool,
    blur: bool,
    icon: PathBuf,
    resizable: bool,
    max_size: Option<(u32, u32)>,
    fullscreen: bool,
    maximized: bool,
    pub(crate) use_vsync: bool,
    theme: Option<Theme>,
}

impl EngineAttributes {
    /// creates new engine attributes (enigine builder with default values)
    pub fn new() -> Self {
        Self {
            title: WIN_TITLE,
            fps_cap: Some(300f64),
            size: (MIN_WIN_WIDTH, MIN_WIN_HEIGHT),
            min_size: None,
            enforced_ratio: None,
            transparent: false,
            blur: false,
            icon: PathBuf::from(get_image_path("icon.ico")),
            resizable: true,
            max_size: None,
            fullscreen: false,
            maximized: false,
            use_vsync: false,
            theme: None,
        }
    }

    /// sets a default window title
    pub fn with_title(mut self, title: &'static str) -> Self {
        self.title = title;
        self
    }

    /// sets an optional fps cap for the rendering process (default is 300)
    pub fn with_fps_cap(mut self, cap: Option<f64>) -> Self {
        self.fps_cap = cap;
        self
    }

    /// sets the width and height for the window (default is 800 x 450) (logical pixels)
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = (width, height);
        self
    }

    /// sets an optional minimum size for the window (default is None)
    pub fn with_min_size(mut self, size: Option<(u32, u32)>) -> Self {
        self.min_size = size;
        self
    }

    /// sets an optional window ratio (width/height) to keep at all times during resizes of the window (default is None)
    pub fn with_fixed_ratio(mut self, ratio: Option<f32>) -> Self {
        self.enforced_ratio = ratio;
        self
    }

    /// sets the window as transparent (default is false)
    pub fn with_transparent(mut self, flag: bool) -> Self {
        self.transparent = flag;
        self
    }

    /// sets the window background as blurred (default is false)
    pub fn with_blur(mut self, flag: bool) -> Self {
        self.blur = flag;
        self
    }

    /// sets the window icon from a file (must be .ico) (only works on windows)
    pub fn with_icon(mut self, path: PathBuf) -> Self {
        self.icon = path;
        self
    }

    /// sets the window icon from a file (must be .ico) (only works on windows)
    pub fn with_resizable(mut self, flag: bool) -> Self {
        self.resizable = flag;
        self
    }

    /// sets the windows' optional maximum size (default is None)
    pub fn with_max_size(mut self, size: Option<(u32, u32)>) -> Self {
        self.max_size = size;
        self
    }

    /// sets the window as fullscreen (default is false)
    pub fn with_fullscreen(mut self, flag: bool) -> Self {
        self.fullscreen = flag;
        self
    }

    /// sets the window as maximized (default is false)
    pub fn with_maximized(mut self, flag: bool) -> Self {
        self.maximized = flag;
        self
    }

    /// enables/disables vsync (default is false)
    pub fn with_vsync(mut self, flag: bool) -> Self {
        self.use_vsync = flag;
        self
    }

    /// sets the theme (light or dark) for the window (default is the System default)
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// builds the actual engine and performs engine
    pub fn build_engine<A: FallingLeafApp>(self) -> Result<Engine<A>, String> {
        if let Some(min_size) = self.min_size {
            if min_size.0 > self.size.0 || min_size.1 > self.size.1 {
                return Err(String::from(
                    "minimun window size can not be bigger than initial size",
                ));
            }
        }
        if let Some(max_size) = self.max_size {
            if max_size.0 < self.size.0 || max_size.1 < self.size.1 {
                return Err(String::from(
                    "maximun window size can not be smaller than initial size",
                ));
            }
        }
        let engine = Engine::new(self);

        Ok(engine)
    }

    /// generate the winit window attributes used for window construction
    pub(crate) fn generate_win_attrs(&self) -> WindowAttributes {
        let mut window_attributes = Window::default_attributes()
            .with_transparent(self.transparent)
            .with_blur(self.blur)
            .with_title(self.title)
            .with_resizable(self.resizable)
            .with_maximized(self.maximized)
            .with_theme(self.theme)
            .with_inner_size(LogicalSize::new(self.size.0, self.size.1));

        if let Some(min_size) = self.min_size {
            window_attributes =
                window_attributes.with_min_inner_size(LogicalSize::new(min_size.0, min_size.1));
        }
        if let Some(max_size) = self.max_size {
            window_attributes =
                window_attributes.with_max_inner_size(LogicalSize::new(max_size.0, max_size.1));
        }
        if self.fullscreen {
            window_attributes =
                window_attributes.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        #[cfg(target_os = "windows")]
        let window_attributes = window_attributes
            .with_window_icon(Some(Icon::from_path(self.icon.as_path(), None).unwrap()));

        window_attributes
    }
}
