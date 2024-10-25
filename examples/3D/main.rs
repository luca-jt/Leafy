//#![windows_subsystem = "windows"]

use crate::app::*;
use env_logger::Env;
use fl_core::engine_builder::EngineAttributes;
use std::error::Error;

mod app;

fn main() -> Result<(), Box<dyn Error>> {
    let env = Env::default()
        .filter_or("LOG_LVL", "trace")
        .write_style_or("LOG_STYLE", "always");
    env_logger::init_from_env(env); // only for testing purposes (not necessary)

    let app = App::new();
    let mut engine = EngineAttributes::new()
        .with_min_size(Some((400, 225)))
        .with_transparent(true)
        .with_fixed_ratio(Some(16.0 / 9.0))
        .build_engine()
        .unwrap();

    engine.run(app)
}
