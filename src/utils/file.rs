use crate::glm;
use gl::types::GLuint;
use stb_image::image::{load_with_depth, Image, LoadResult};
use std::path::Path;

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
pub(crate) const BATCH_B_FRAG: &str = include_str!(shader_path!("batch/basic.frag"));
pub(crate) const BATCH_B_VERT: &str = include_str!(shader_path!("batch/basic.vert"));
pub(crate) const BATCH_PT_FRAG: &str = include_str!(shader_path!("batch/passthrough.frag"));
pub(crate) const BATCH_PT_VERT: &str = include_str!(shader_path!("batch/passthrough.vert"));
pub(crate) const INSTANCE_B_FRAG: &str = include_str!(shader_path!("instance/basic.frag"));
pub(crate) const INSTANCE_B_VERT: &str = include_str!(shader_path!("instance/basic.vert"));
pub(crate) const INSTANCE_PT_FRAG: &str = include_str!(shader_path!("instance/passthrough.frag"));
pub(crate) const INSTANCE_PT_VERT: &str = include_str!(shader_path!("instance/passthrough.vert"));
pub(crate) const INSTANCE_SHADOW_FRAG: &str = include_str!(shader_path!("instance/shadow.frag"));
pub(crate) const INSTANCE_SHADOW_VERT: &str = include_str!(shader_path!("instance/shadow.vert"));
pub(crate) const BATCH_SHADOW_FRAG: &str = include_str!(shader_path!("batch/shadow.frag"));
pub(crate) const BATCH_SHADOW_VERT: &str = include_str!(shader_path!("batch/shadow.vert"));
pub(crate) const SKYBOX_VERT: &str = include_str!(shader_path!("skybox.vert"));
pub(crate) const SKYBOX_FRAG: &str = include_str!(shader_path!("skybox.frag"));
pub(crate) const SCREEN_VERT: &str = include_str!(shader_path!("screen.vert"));
pub(crate) const SCREEN_FRAG: &str = include_str!(shader_path!("screen.frag"));
pub(crate) const SPRITE_VERT: &str = include_str!(shader_path!("sprite.vert"));
pub(crate) const SPRITE_FRAG: &str = include_str!(shader_path!("sprite.frag"));

// optional included meshes
pub(crate) const TRIANGLE_MESH: &[u8] = include_bytes!(model_path!("triangle.obj"));
pub(crate) const PLANE_MESH: &[u8] = include_bytes!(model_path!("plane.obj"));
pub(crate) const CUBE_MESH: &[u8] = include_bytes!(model_path!("cube.obj"));
pub(crate) const SPHERE_MESH: &[u8] = include_bytes!(model_path!("sphere.obj"));
pub(crate) const TORUS_MESH: &[u8] = include_bytes!(model_path!("torus.obj"));

#[rustfmt::skip]
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

#[rustfmt::skip]
pub(crate) const SCREEN_TRIANGLE_VERTICES: [f32; 15] = [
    -1.0, -1.0, 0.0, 0.0, 0.0,
     3.0, -1.0, 0.0, 2.0, 0.0,
    -1.0,  3.0, 0.0, 0.0, 2.0
];

pub(crate) const SPRITE_PLANE_VERTICES: [glm::Vec3; 4] = [
    glm::Vec3::new(-0.5, -0.5, 0.0),
    glm::Vec3::new(0.5, 0.5, 0.0),
    glm::Vec3::new(-0.5, 0.5, 0.0),
    glm::Vec3::new(0.5, -0.5, 0.0),
];
pub(crate) const SPRITE_PLANE_UVS: [glm::Vec2; 4] = [
    glm::Vec2::new(0.0, 1.0),
    glm::Vec2::new(1.0, 0.0),
    glm::Vec2::new(0.0, 0.0),
    glm::Vec2::new(1.0, 1.0),
];
pub(crate) const SPRITE_PLANE_INDICES: [GLuint; 6] = [0, 1, 2, 0, 3, 1];

// audio data
pub(crate) const HRTF_SPHERE: &[u8] = include_bytes!(audio_path!("IRC_1002_C.bin"));

/// loads an u8 image (probably PNG) using stb image from a given path
pub fn stbi_load_u8_rgba(file_path: impl AsRef<Path>) -> Option<Image<u8>> {
    match load_with_depth(file_path, 4, false) {
        LoadResult::ImageU8(im) => Some(im),
        _ => None,
    }
}
