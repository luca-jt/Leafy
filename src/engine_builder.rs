use crate::internal_prelude::*;
use winit::dpi::PhysicalSize;
use winit::window::{Fullscreen, Theme, Window, WindowAttributes};

#[cfg(target_os = "windows")]
use winit::platform::windows::IconExtWindows;
#[cfg(target_os = "windows")]
use winit::window::Icon;

/// Builder structure for the ``Engine`` that allows for easy modification of intial parameters. You create this before running the engine and you can use all of the ``.with_...()`` functions to specify the start configuration of the engine.
pub struct EngineAttributes {
    title: &'static str,
    pub(crate) fps_cap: Option<f64>,
    pub(crate) bg_fps_cap: Option<f64>,
    size: (u32, u32),
    min_size: Option<(u32, u32)>,
    pub(crate) enforced_ratio: Option<f32>,
    transparent: bool,
    icon: AppIcon,
    resizable: bool,
    max_size: Option<(u32, u32)>,
    fullscreen: bool,
    maximized: bool,
    pub(crate) use_vsync: bool,
    theme: Option<Theme>,
}

impl EngineAttributes {
    /// Creates new ``EngineAttributes`` with default parameters.
    pub fn new() -> Self {
        Self {
            title: WIN_TITLE,
            fps_cap: None,
            bg_fps_cap: None,
            size: (DEFAULT_WIN_WIDTH, DEFAULT_WIN_HEIGHT),
            min_size: None,
            enforced_ratio: None,
            transparent: false,
            icon: AppIcon::Default,
            resizable: true,
            max_size: None,
            fullscreen: false,
            maximized: false,
            use_vsync: true,
            theme: None,
        }
    }

    /// Sets a default window title.
    pub fn with_title(mut self, title: &'static str) -> Self {
        self.title = title;
        self
    }

    /// Sets an optional FPS cap for the rendering process (default is ``None`` because VSYNC is enabled).
    pub fn with_fps_cap(mut self, cap: Option<f64>) -> Self {
        self.fps_cap = cap;
        self
    }

    /// Sets an optional FPS cap for the rendering process when the app is not in focus (default is ``None``).
    pub fn with_bg_fps_cap(mut self, cap: Option<f64>) -> Self {
        self.bg_fps_cap = cap;
        self
    }

    /// Sets the width and height for the window (default is 800 x 450) in physical size.
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = (width, height);
        self
    }

    /// Sets an optional minimum size (width, height) for the window (default is ``None``).
    pub fn with_min_size(mut self, size: Option<(u32, u32)>) -> Self {
        self.min_size = size;
        self
    }

    /// Sets an optional window ratio (width/height) to keep at all times during resizes of the window (default is ``None``).
    pub fn with_fixed_ratio(mut self, ratio: Option<f32>) -> Self {
        self.enforced_ratio = ratio;
        self
    }

    /// Sets the window as transparent (default is ``false``). This enables to render to the background with an alpha value.
    pub fn with_transparent(mut self, flag: bool) -> Self {
        self.transparent = flag;
        self
    }

    /// Sets the window icon from a file (must be a ``.ico`` file) (only works on windows).
    pub fn with_icon(mut self, path: PathBuf) -> Self {
        self.icon = AppIcon::Custom(path);
        self
    }

    /// Sets the window to be resizable (default is ``true``).
    pub fn with_resizable(mut self, flag: bool) -> Self {
        self.resizable = flag;
        self
    }

    /// Sets the window's optional maximum size (width, height) (default is ``None``).
    pub fn with_max_size(mut self, size: Option<(u32, u32)>) -> Self {
        self.max_size = size;
        self
    }

    /// Sets the window as fullscreen (default is ``false``).
    pub fn with_fullscreen(mut self, flag: bool) -> Self {
        self.fullscreen = flag;
        self
    }

    // Sets the window as maximized (default is ``false``).
    pub fn with_maximized(mut self, flag: bool) -> Self {
        self.maximized = flag;
        self
    }

    /// Enables/disables VSYNC (default is ``true``) (disabling this and not setting an FPS cap will set the frame rate to unlimited).
    pub fn with_vsync(mut self, flag: bool) -> Self {
        self.use_vsync = flag;
        self
    }

    /// Sets the theme (light or dark) for the window (default is the system default).
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Builds the actual engine and performs compatibility checks.
    pub fn build_engine<A: FallingLeafApp>(self) -> Result<Engine<A>, String> {
        if let Some(min_size) = self.min_size {
            if min_size.0 > self.size.0 || min_size.1 > self.size.1 {
                return Err(String::from(
                    "Minimun window size can not be bigger than initial size.",
                ));
            }
        }
        if let Some(max_size) = self.max_size {
            if max_size.0 < self.size.0 || max_size.1 < self.size.1 {
                return Err(String::from(
                    "Maximun window size can not be smaller than initial size.",
                ));
            }
        }
        if self.fullscreen && self.maximized {
            return Err(String::from(
                "The game can not be maximimzed and in fullscreen mode at the same time.",
            ));
        }
        let engine = Engine::new(self);

        Ok(engine)
    }

    /// Generate the winit window attributes used for window construction.
    pub(crate) fn generate_win_attrs(&self) -> WindowAttributes {
        let mut window_attributes = Window::default_attributes()
            .with_transparent(self.transparent)
            .with_title(self.title)
            .with_resizable(self.resizable)
            .with_maximized(self.maximized)
            .with_theme(self.theme)
            .with_inner_size(PhysicalSize::new(self.size.0, self.size.1))
            .with_fullscreen(self.fullscreen.then_some(Fullscreen::Borderless(None)));

        if let Some(min_size) = self.min_size {
            window_attributes =
                window_attributes.with_min_inner_size(PhysicalSize::new(min_size.0, min_size.1));
        }
        if let Some(max_size) = self.max_size {
            window_attributes =
                window_attributes.with_max_inner_size(PhysicalSize::new(max_size.0, max_size.1));
        }

        #[cfg(target_os = "windows")]
        let window_attributes = window_attributes.with_window_icon(Some(self.icon.generate_icon()));

        window_attributes
    }
}

impl Default for EngineAttributes {
    fn default() -> Self {
        Self::new()
    }
}

enum AppIcon {
    Default,
    Custom(PathBuf),
}

impl AppIcon {
    #[cfg(target_os = "windows")]
    fn generate_icon(&self) -> Icon {
        match self {
            AppIcon::Default => Icon::from_resource(32512, None).unwrap(),
            AppIcon::Custom(path_buf) => Icon::from_path(path_buf.as_path(), None).unwrap(),
        }
    }
}
