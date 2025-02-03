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
        .with_transparent(true)
        .with_title("Conway's Game of Life")
        .with_size(800, 800)
        .with_resizable(false)
        .build_engine()
        .unwrap();

    engine.run(app)
}
