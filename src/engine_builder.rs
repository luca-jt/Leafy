pub struct EngineBuilder {
    title: &'static str,
    fps_cap: Option<f64>,
    size: (u32, u32),
    min_size: Option<(u32, u32)>,
    inv_ratio: f32,
    keep_ratio: bool,
    transparent: bool,
    blur: bool,
    icon: &'static str,
}

impl EngineBuilder {}
