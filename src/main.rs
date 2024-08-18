#![windows_subsystem = "windows"]

use crate::app::*;

mod app;

fn main() {
    let mut app = App::init();
    app.run();
}
