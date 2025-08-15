use leafy::prelude::*;

pub struct App;

impl LeafyApp for App {
    fn init(&mut self, _engine: &Engine<Self>) {}

    fn on_frame_update(&mut self, _engine: &Engine<Self>) {}
}
