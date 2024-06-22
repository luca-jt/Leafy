use crate::utils::file::get_texture_path;
use gl::types::*;
use nalgebra_glm as glm;
use stb_image::image::{Image, LoadResult};
use std::collections::HashMap;

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

#[derive(Default, Clone)]
#[repr(C)]
/// data for a single vertex
pub struct Vertex {
    pub position: glm::Vec3,
    pub color: glm::Vec3,
    pub uv_coords: glm::Vec2,
    pub normal: glm::Vec3,
    pub tex_index: GLfloat,
}

/// holds the texture ID's for the App
pub struct TextureMap {
    textures: HashMap<String, GLuint>,
}

impl TextureMap {
    /// creates a new texture map
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    /// adds a texture from file
    pub fn add_texture(&mut self, name: &str, file: &str) {
        self.textures.insert(name.to_string(), load_texture(file));
    }

    /// deletes a stored texture
    pub fn delete_texture(&mut self, name: &str) {
        let deleted = self.textures.remove(name).expect("texture not stored");
        unsafe {
            gl::DeleteTextures(1, &deleted);
        }
    }

    /// yields a texture id for given name
    pub fn get_tex_id(&self, name: &str) -> GLuint {
        *self.textures.get(name).unwrap()
    }
}

impl Drop for TextureMap {
    fn drop(&mut self) {
        unsafe {
            for (_, texture) in self.textures.iter() {
                gl::DeleteTextures(1, texture);
            }
        }
    }
}
