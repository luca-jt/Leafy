use stb_image::image::{load_from_memory_with_depth, load_with_depth, Image, LoadResult};
use std::path::Path;

// directory paths
macro_rules! batch_shader_path {
    ($file:literal) => {
        concat!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/batch/"),
            $file
        )
    };
}

macro_rules! instance_shader_path {
    ($file:literal) => {
        concat!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/shaders/instance/"),
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
pub(crate) const BATCH_B_FRAG: &str = include_str!(batch_shader_path!("basic.frag"));
pub(crate) const BATCH_B_VERT: &str = include_str!(batch_shader_path!("basic.vert"));
pub(crate) const BATCH_PT_FRAG: &str = include_str!(batch_shader_path!("passthrough.frag"));
pub(crate) const BATCH_PT_VERT: &str = include_str!(batch_shader_path!("passthrough.vert"));
pub(crate) const INSTANCE_B_FRAG: &str = include_str!(instance_shader_path!("basic.frag"));
pub(crate) const INSTANCE_B_VERT: &str = include_str!(instance_shader_path!("basic.vert"));
pub(crate) const INSTANCE_PT_FRAG: &str = include_str!(instance_shader_path!("passthrough.frag"));
pub(crate) const INSTANCE_PT_VERT: &str = include_str!(instance_shader_path!("passthrough.vert"));
pub(crate) const INSTANCE_SHADOW_FRAG: &str = include_str!(instance_shader_path!("shadow.frag"));
pub(crate) const INSTANCE_SHADOW_VERT: &str = include_str!(instance_shader_path!("shadow.vert"));
pub(crate) const BATCH_SHADOW_FRAG: &str = include_str!(batch_shader_path!("shadow.frag"));
pub(crate) const BATCH_SHADOW_VERT: &str = include_str!(batch_shader_path!("shadow.vert"));

// optional included meshes
pub(crate) const TRIANGLE_MESH: &[u8] = include_bytes!(model_path!("triangle.obj"));
pub(crate) const PLANE_MESH: &[u8] = include_bytes!(model_path!("plane.obj"));
pub(crate) const CUBE_MESH: &[u8] = include_bytes!(model_path!("cube.obj"));
pub(crate) const CONE_MESH: &[u8] = include_bytes!(model_path!("cone.obj"));
pub(crate) const CYLINDER_MESH: &[u8] = include_bytes!(model_path!("cylinder.obj"));
pub(crate) const SPHERE_MESH: &[u8] = include_bytes!(model_path!("sphere.obj"));
pub(crate) const TORUS_MESH: &[u8] = include_bytes!(model_path!("torus.obj"));

// audio data
pub(crate) const HRTF_SPHERE: &[u8] = include_bytes!(audio_path!("IRC_1002_C.bin"));

/// loads an u8 image (probably PNG) using stb image from a given path
pub fn stbi_load_u8_rgba(file_path: impl AsRef<Path>) -> Option<Image<u8>> {
    match load_with_depth(file_path, 4, false) {
        LoadResult::ImageU8(im) => Some(im),
        _ => None,
    }
}

/// loads an u8 image (probably PNG) using stb image from bytes
pub fn stbi_load_u8_rgba_from_bytes(bytes: &[u8]) -> Option<Image<u8>> {
    match load_from_memory_with_depth(bytes, 4, false) {
        LoadResult::ImageU8(im) => Some(im),
        _ => None,
    }
}
