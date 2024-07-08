use crate::rendering::batch_renderer::BatchRenderer;
use stb_image::image::Image;

pub struct SpriteRenderer {
    batch_renderer: BatchRenderer,
    sprite_sheet: Image<u8>,
}

impl SpriteRenderer {
    /*pub fn new() -> Self {
        Self {}
    }*/
    // TODO: render with sprite index + pixel position + scale

    pub fn init(&mut self) {}
    pub fn end(&mut self) {}
}
