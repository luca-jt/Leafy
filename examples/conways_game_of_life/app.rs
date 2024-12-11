use falling_leaf::ecs::component::utils::Color32;
use falling_leaf::engine::{Engine, FallingLeafApp};

pub struct App;

impl FallingLeafApp for App {
    fn init(&mut self, engine: &Engine<Self>) {
        engine
            .rendering_system_mut()
            .set_gl_clearcolor(Color32::TRANSPARENT);
    }

    fn on_frame_update(&mut self, _engine: &Engine<Self>) {}
}
