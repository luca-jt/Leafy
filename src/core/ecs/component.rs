use gl::types::GLuint;
use nalgebra_glm as glm;
use std::any::Any;
use MeshAttribute::*;

/// defines what can be a component for an entity
pub trait Component: Any + 'static {}

impl<T> Component for T where T: Any + 'static {}

pub struct Scale(f32);

impl From<f32> for Scale {
    fn from(value: f32) -> Self {
        Scale(value)
    }
}

pub struct Position(glm::Vec3);

pub struct Quaternion(glm::Vec4); // TODO: nutzung

pub struct Velocity(glm::Vec3);

pub struct Acceleration(glm::Vec3);

/// binds all of the motion-specific data together
pub enum MotionState {
    Moving(Velocity, Acceleration),
    Fixed,
}

impl Default for MotionState {
    fn default() -> Self {
        MotionState::Moving(Velocity::zeros(), Acceleration::zeros())
    }
}

/// efficient 32bit color representation
#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
pub struct Color32([u8; 4]);

impl Color32 {
    pub const WHITE: Self = Self::from_rgb(255, 255, 255);
    pub const BLACK: Self = Self::from_rgb(0, 0, 0);
    pub const TRANSPARENT: Self = Self([0, 0, 0, 0]);
    pub const RED: Self = Self::from_rgb(255, 0, 0);
    pub const GREEN: Self = Self::from_rgb(0, 255, 0);
    pub const BLUE: Self = Self::from_rgb(0, 0, 255);

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b, 255])
    }

    pub fn r(&self) -> u8 {
        self.0[0]
    }

    pub fn g(&self) -> u8 {
        self.0[1]
    }

    pub fn b(&self) -> u8 {
        self.0[2]
    }

    pub fn a(&self) -> u8 {
        self.0[3]
    }

    /// converts to a float rgba vector
    pub fn to_vec4(&self) -> glm::Vec4 {
        let r = self.r() as f32 / 255.0;
        let g = self.g() as f32 / 255.0;
        let b = self.b() as f32 / 255.0;
        let a = self.a() as f32 / 255.0;

        glm::Vec4::new(r, g, b, a)
    }
}

/// all of the known mesh types
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub enum MeshType {
    Sphere,
    Cube,
    Plane,
}

/// wether or not a mesh is colored or textured
#[derive(Copy, Clone, PartialEq)]
pub enum MeshAttribute {
    Textured(GLuint),
    Colored(Color32),
}

impl MeshAttribute {
    /// returns the texture id if present
    pub fn tex_id(&self) -> Option<GLuint> {
        match self {
            Textured(id) => Some(*id),
            Colored(_) => None,
        }
    }

    /// returns the color if present
    pub fn color(&self) -> Option<Color32> {
        match self {
            Textured(_) => None,
            Colored(color) => Some(*color),
        }
    }
}
