use crate::ecs::component::Acceleration;

pub const WIN_TITLE: &str = "Falling Leaf";
pub const MIN_WIN_WIDTH: u32 = 800;
pub const MIN_WIN_HEIGHT: u32 = 450;
pub const INV_WIN_RATIO: f32 = 9.0 / 16.0;

pub const FPS_CAP: f64 = 300.0;

pub const MAX_TEXTURE_COUNT: usize = 32;

pub const G: Acceleration = Acceleration::new(0.0, -9.81, 0.0);
