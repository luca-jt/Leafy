use crate::rendering::batch_renderer::BatchRenderer;
use stb_image::image::Image;

pub(crate) struct SpriteRenderer {
    batch_renderer: BatchRenderer,
    sprite_sheet: Image<u8>,
}

impl SpriteRenderer {
    /*pub fn new() -> Self {
        Self {}
    }*/

    // render with sprite index + pixel position + scale

    pub(crate) fn init(&mut self) {}
    pub(crate) fn end(&mut self) {}
}
