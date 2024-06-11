#![windows_subsystem = "windows"]

use crate::app::*;

mod app;
mod audio;
mod ecs;
mod rendering;
mod state;
mod systems;
mod utils;

fn main() {
    let mut app = App::new();
    app.run();
}
