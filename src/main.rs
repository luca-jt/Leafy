#![windows_subsystem = "windows"]

use crate::app::*;
use fl_core::engine::Engine;

mod app;

fn main() {
    let mut app = App::new();
    let mut engine = Engine::new();
    engine.run(&mut app);
}
