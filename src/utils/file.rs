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

macro_rules! material_path {
    ($file:literal) => {
        concat!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/materials/"),
            $file
        )
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
pub(crate) const BATCH_B_FRAG: &'static str = include_str!(shader_path!("batch_basic.frag"));
pub(crate) const BATCH_B_VERT: &'static str = include_str!(shader_path!("batch_basic.vert"));
pub(crate) const BATCH_PT_FRAG: &'static str = include_str!(shader_path!("batch_passthrough.frag"));
pub(crate) const BATCH_PT_VERT: &'static str = include_str!(shader_path!("batch_passthrough.vert"));
pub(crate) const INST_B_FRAG: &'static str = include_str!(shader_path!("inst_basic.frag"));
pub(crate) const INST_B_VERT: &'static str = include_str!(shader_path!("inst_basic.vert"));
pub(crate) const INST_PT_FRAG: &'static str = include_str!(shader_path!("inst_passthrough.frag"));
pub(crate) const INST_PT_VERT: &'static str = include_str!(shader_path!("inst_passthrough.vert"));
pub(crate) const SHADOW_FRAG: &'static str = include_str!(shader_path!("shadow.frag"));
pub(crate) const SHADOW_VERT: &'static str = include_str!(shader_path!("shadow.vert"));

// optional mesh features
#[cfg(feature = "triangle_mesh")]
pub(crate) const TRIANGLE_MESH: &[u8] = include_bytes!(model_path!("triangle.obj"));
#[cfg(not(feature = "triangle_mesh"))]
pub(crate) const TRIANGLE_MESH: &[u8] = &[0];

#[cfg(feature = "plane_mesh")]
pub(crate) const PLANE_MESH: &[u8] = include_bytes!(model_path!("plane.obj"));
#[cfg(not(feature = "plane_mesh"))]
pub(crate) const PLANE_MESH: &[u8] = &[0];

#[cfg(feature = "cube_mesh")]
pub(crate) const CUBE_MESH: &[u8] = include_bytes!(model_path!("cube.obj"));
#[cfg(not(feature = "cube_mesh"))]
pub(crate) const CUBE_MESH: &[u8] = &[0];

#[cfg(feature = "cone_mesh")]
pub(crate) const CONE_MESH: &[u8] = include_bytes!(model_path!("cone.obj"));
#[cfg(not(feature = "cone_mesh"))]
pub(crate) const CONE_MESH: &[u8] = &[0];

#[cfg(feature = "cylinder_mesh")]
pub(crate) const CYLINDER_MESH: &[u8] = include_bytes!(model_path!("cylinder.obj"));
#[cfg(not(feature = "cylinder_mesh"))]
pub(crate) const CYLINDER_MESH: &[u8] = &[0];

#[cfg(feature = "sphere_mesh")]
pub(crate) const SPHERE_MESH: &[u8] = include_bytes!(model_path!("sphere.obj"));
#[cfg(not(feature = "sphere_mesh"))]
pub(crate) const SPHERE_MESH: &[u8] = &[0];

#[cfg(feature = "torus_mesh")]
pub(crate) const TORUS_MESH: &[u8] = include_bytes!(model_path!("torus.obj"));
#[cfg(not(feature = "torus_mesh"))]
pub(crate) const TORUS_MESH: &[u8] = &[0];

// audio data
pub(crate) const HRTF_SPHERE: &[u8] = include_bytes!(audio_path!("IRC_1002_C.bin"));

// texture data
pub(crate) const WALL_TEXTURE: &[u8] = include_bytes!(texture_path!("wall.png"));
pub(crate) const ICE_TEXTURE: &[u8] = include_bytes!(texture_path!("ice.png"));
pub(crate) const SAND_TEXTURE: &[u8] = include_bytes!(texture_path!("sand.png"));
