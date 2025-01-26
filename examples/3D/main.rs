use crate::app::*;
use env_logger::Env;
use falling_leaf::engine_builder::EngineAttributes;
use std::error::Error;

mod app;

fn main() -> Result<(), Box<dyn Error>> {
    let env = Env::default()
        .filter_or("LOG_LVL", "info")
        .write_style_or("LOG_STYLE", "always");
    env_logger::init_from_env(env); // only for testing purposes (not necessary)

    let app = App::new();
    let mut engine = EngineAttributes::new()
        .with_min_size(Some((400, 225)))
        .with_transparent(true)
        .with_title("3D")
        .with_fixed_ratio(Some(16.0 / 9.0))
        .with_bg_fps_cap(Some(30.0))
        .build_engine()
        .unwrap();

    engine.run(app)
}
