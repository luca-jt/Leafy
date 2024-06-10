#![windows_subsystem = "windows"]

use crate::app::*;

mod app;
mod audio;
mod events;
mod rendering;
mod state;
mod utils;

fn main() {
    let mut app = App::new();
    app.run();
}
