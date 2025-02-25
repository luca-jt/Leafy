use falling_leaf::prelude::*;

pub struct App;

impl FallingLeafApp for App {
    fn init(&mut self, _engine: &Engine<Self>) {}

    fn on_frame_update(&mut self, _engine: &Engine<Self>) {}
}
