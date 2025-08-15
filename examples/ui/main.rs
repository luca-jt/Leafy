use crate::app::*;
use env_logger::Env;
use leafy::engine_builder::EngineAttributes;
use std::error::Error;

mod app;

fn main() -> Result<(), Box<dyn Error>> {
    let env = Env::default()
        .filter_or("LOG_LVL", "debug")
        .write_style_or("LOG_STYLE", "always");
    env_logger::init_from_env(env); // only for testing purposes (not necessary)

    let app = App;
    let mut engine = EngineAttributes::new()
        .with_title("UI")
        .build_engine()
        .unwrap();

    engine.run(app)
}
