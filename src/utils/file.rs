use gl::types::*;
use stb_image::image::{Image, LoadResult};
use std::env::current_dir;

/// yields the full path of any asset file located in ./assets/file_path
pub fn get_asset_path(dir_path: &str) -> String {
    let full_path = current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
        .replace("\\", "/")
        + "/assets/"
        + dir_path;

    return full_path;
}

/// yields audio file path
pub fn get_audio_path(file_name: &str) -> String {
    get_asset_path("audio/") + file_name
}

/// yields image file path
pub fn get_image_path(file_name: &str) -> String {
    get_asset_path("images/") + file_name
}

/// yields texture file path
fn get_texture_path(file_name: &str) -> String {
    get_asset_path("textures/") + file_name
}

/// yields model file path
pub fn get_model_path(file_name: &str) -> String {
    get_asset_path("models/") + file_name
}

/// yields shader file path
pub fn get_shader_path(file_name: &str) -> String {
    get_asset_path("shaders/") + file_name
}

/// loads an opengl texture
pub fn load_texture(file_name: &str) -> GLuint {
    let mut tex_id = 0;

    let texture: Image<u8>;
    match stb_image::image::load_with_depth(get_texture_path(file_name), 4, false) {
        LoadResult::ImageU8(im) => {
            texture = im;
        }
        _ => {
            panic!("error reading texture")
        }
    }

    // generate gl texture
    unsafe {
        gl::GenTextures(1, &mut tex_id);
        gl::BindTexture(gl::TEXTURE_2D, tex_id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as GLint,
            texture.width as GLint,
            texture.height as GLint,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            texture.data.as_ptr() as *const GLvoid,
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    tex_id
}

// TODO: parser for obj files
