#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::internal_prelude::*;
use stb_image::image::{load_with_depth, Image, LoadResult};

// directory paths
macro_rules! shader_path {
    ($file:literal) => {
        concat!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/"),
            $file
        )
    };
}

macro_rules! audio_path {
    ($file:literal) => {
        concat!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/audio/"), $file)
    };
}

macro_rules! model_path {
    ($file:literal) => {
        concat!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models/"),
            $file
        )
    };
}

// shader files
pub(crate) const BASIC_FRAG: &str = include_str!(shader_path!("basic.frag"));
pub(crate) const BASIC_VERT: &str = include_str!(shader_path!("basic.vert"));
pub(crate) const PASSTHROUGH_FRAG: &str = include_str!(shader_path!("passthrough.frag"));
pub(crate) const PASSTHROUGH_VERT: &str = include_str!(shader_path!("passthrough.vert"));
pub(crate) const SHADOW_FRAG: &str = include_str!(shader_path!("shadow.frag"));
pub(crate) const SHADOW_VERT: &str = include_str!(shader_path!("shadow.vert"));
pub(crate) const CUBE_SHADOW_VERT: &str = include_str!(shader_path!("cube_shadow.vert"));
pub(crate) const CUBE_SHADOW_FRAG: &str = include_str!(shader_path!("cube_shadow.frag"));
pub(crate) const CUBE_SHADOW_GEOM: &str = include_str!(shader_path!("cube_shadow.geom"));
pub(crate) const SKYBOX_VERT: &str = include_str!(shader_path!("skybox.vert"));
pub(crate) const SKYBOX_FRAG: &str = include_str!(shader_path!("skybox.frag"));
pub(crate) const SCREEN_VERT: &str = include_str!(shader_path!("screen.vert"));
pub(crate) const SCREEN_FRAG: &str = include_str!(shader_path!("screen.frag"));
pub(crate) const SPRITE_VERT: &str = include_str!(shader_path!("sprite.vert"));
pub(crate) const SPRITE_FRAG: &str = include_str!(shader_path!("sprite.frag"));
pub(crate) const BLUR_VERT: &str = include_str!(shader_path!("blur.vert"));
pub(crate) const BLUR_FRAG: &str = include_str!(shader_path!("blur.frag"));
pub(crate) const OUTLINE_VERT: &str = include_str!(shader_path!("outline.vert"));
pub(crate) const OUTLINE_FRAG: &str = include_str!(shader_path!("outline.frag"));

// included meshes
pub(crate) const TRIANGLE_MESH: &[u8] = include_bytes!(model_path!("triangle.obj"));
pub(crate) const PLANE_MESH: &[u8] = include_bytes!(model_path!("plane.obj"));
pub(crate) const CUBE_MESH: &[u8] = include_bytes!(model_path!("cube.obj"));

pub(crate) const SKYBOX_VERTICES: [f32; 108] = [
    -1.0,  1.0, -1.0,
    -1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,

    -1.0, -1.0,  1.0,
    -1.0, -1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0,  1.0,
    -1.0, -1.0,  1.0,

     1.0, -1.0, -1.0,
     1.0, -1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0, -1.0,
     1.0, -1.0, -1.0,

    -1.0, -1.0,  1.0,
    -1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0, -1.0,  1.0,
    -1.0, -1.0,  1.0,

    -1.0,  1.0, -1.0,
     1.0,  1.0, -1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
    -1.0,  1.0,  1.0,
    -1.0,  1.0, -1.0,

    -1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
     1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
     1.0, -1.0,  1.0
];

pub(crate) const SCREEN_TRIANGLE_VERTICES: [f32; 15] = [
    -1.0, -1.0, 0.0, 0.0, 0.0,
     3.0, -1.0, 0.0, 2.0, 0.0,
    -1.0,  3.0, 0.0, 0.0, 2.0
];

pub(crate) const SPRITE_PLANE_VERTICES: [Vec3; 4] = [
    Vec3::new(-0.5, -0.5, 0.0),
    Vec3::new(0.5, 0.5, 0.0),
    Vec3::new(-0.5, 0.5, 0.0),
    Vec3::new(0.5, -0.5, 0.0),
];
pub(crate) const SPRITE_PLANE_UVS: [Vec2; 4] = [
    Vec2::new(0.0, 1.0),
    Vec2::new(1.0, 0.0),
    Vec2::new(0.0, 0.0),
    Vec2::new(1.0, 1.0),
];
pub(crate) const SPRITE_PLANE_INDICES: [GLuint; 6] = [0, 1, 2, 0, 3, 1];

// audio data
pub(crate) const HRTF_SPHERE: &[u8] = include_bytes!(audio_path!("IRC_1002_C.bin"));

/// Loads an ``Image<u8>`` (probably from a ``.png`` file) with an alpha channel using stb_image from a given path.
pub fn stbi_load_u8_rgba(file_path: impl AsRef<Path>) -> Option<Image<u8>> {
    match load_with_depth(file_path, 4, false) {
        LoadResult::ImageU8(im) => Some(im),
        _ => None,
    }
}
