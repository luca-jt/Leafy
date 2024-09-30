use crate::engine::Engine;
use crate::utils::constants::*;

pub struct EngineBuilder {
    title: &'static str,
    fps_cap: Option<f64>,
    size: (u32, u32),
    min_size: Option<(u32, u32)>,
    inv_ratio: Option<f32>,
    transparent: bool,
    blur: bool,
    icon: &'static str,
}

impl EngineBuilder {
    pub fn new() -> Self {
        Self {
            title: WIN_TITLE,
            fps_cap: Some(300f64),
            size: (MIN_WIN_WIDTH, MIN_WIN_HEIGHT),
            min_size: None,
            inv_ratio: Some(INV_WIN_RATIO),
            transparent: false,
            blur: false,
            icon: "icon.ico",
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

    /// sets the width and height for the window (default is 800 x 450)
    pub fn with_size(mut self, size: (u32, u32)) -> Self {
        self.size = size;
        self
    }

    /// sets an optional minimum size for the window (default is None)
    pub fn with_min_size(mut self, size: Option<(u32, u32)>) -> Self {
        self.min_size = size;
        self
    }

    /// sets an optional window ratio (height/width) to keep at all times during resizes of the window (default is 9/16)
    pub fn with_fixed_ratio(mut self, ratio: Option<f32>) -> Self {
        self.inv_ratio = ratio;
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

    /// sets the window icon from a file (must be .ico)
    pub fn with_icon(mut self, path: &'static str) -> Self {
        self.icon = path;
        self
    }

    pub fn build(self) -> Result<Engine, String> {
        // check for window size compatibility with minimum
        todo!()
    }
}
