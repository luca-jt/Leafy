#![windows_subsystem = "windows"]

use crate::app::*;
use fl_core::test_link;

mod app;

fn main() {
    test_link();
    let mut app = App::new();
    app.run();
}
