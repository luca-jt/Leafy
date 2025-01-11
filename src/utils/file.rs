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

macro_rules! texture_path {
    ($file:literal) => {
        concat!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/textures/"),
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

// texture data
pub(crate) const WALL_TEXTURE: &[u8] = include_bytes!(texture_path!("wall.png"));
pub(crate) const ICE_TEXTURE: &[u8] = include_bytes!(texture_path!("ice.png"));
pub(crate) const SAND_TEXTURE: &[u8] = include_bytes!(texture_path!("sand.png"));
