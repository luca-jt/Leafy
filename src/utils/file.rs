use std::env::current_dir;

/// yields the full path of any asset file located in ./assets/file_path
fn get_asset_path(dir_path: &str) -> String {
    let full_path = current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
        .replace("\\", "/")
        + "/assets/"
        + dir_path;

    full_path
}

/// yields audio file path
pub fn get_audio_path(file_name: &str) -> String {
    get_asset_path("audio/") + file_name
}

/// yields image file path
pub(crate) fn get_image_path(file_name: &str) -> String {
    get_asset_path("images/") + file_name
}

/// yields texture file path
pub(crate) fn get_texture_path(file_name: &str) -> String {
    get_asset_path("textures/") + file_name
}

/// yields model file path
pub(crate) fn get_model_path(file_name: &str) -> String {
    get_asset_path("models/") + file_name
}

/// yields material file path
pub(crate) fn get_material_path(file_name: &str) -> String {
    get_asset_path("materials/") + file_name
}

/// yields shader file path
pub(crate) fn get_shader_path(file_name: &str) -> String {
    get_asset_path("shaders/") + file_name
}
