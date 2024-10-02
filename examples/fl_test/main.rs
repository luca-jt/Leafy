//#![windows_subsystem = "windows"]

use crate::app::*;
use fl_core::engine_builder::EngineAttributes;
use std::error::Error;

mod app;

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new();
    let mut engine = EngineAttributes::new()
        .with_transparent(true)
        .build_engine()
        .unwrap();

    engine.run(app)
}
