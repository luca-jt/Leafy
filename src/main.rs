#![windows_subsystem = "windows"]

use crate::app::*;
use fl_core::engine::Engine;
use std::error::Error;

mod app;

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new();
    let mut engine = Engine::new();
    engine.run(app)
}
