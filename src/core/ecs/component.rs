use gl::types::GLuint;
use nalgebra_glm as glm;
use std::time::Instant;
use MeshAttribute::*;

/// wrapper struct for an object scaling
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct Scale(pub f32);

impl Into<Scale> for f32 {
    fn into(self) -> Scale {
        Scale(self)
    }
}

impl Default for Scale {
    fn default() -> Self {
        Scale(1.0)
    }
}

/// all data needed for the 3D rendering process
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Renderable {
    pub scale: Scale,
    pub mesh_type: MeshType,
    pub mesh_attribute: MeshAttribute,
}

/// used for object orientation in 3D space
pub type Quaternion = glm::Vec4; // TODO: nutzung

/// position in 3D space
#[derive(Debug, Clone, PartialEq)]
pub struct Position(glm::Vec3);

impl Position {
    /// creates a new position
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Position(glm::Vec3::new(x, y, z))
    }

    /// yields a copy of the stored data
    pub fn data_clone(&self) -> glm::Vec3 {
        self.0.clone()
    }

    /// grants immutable access to the stored data
    pub fn data(&self) -> &glm::Vec3 {
        &self.0
    }

    /// grants mutable access to the stored data
    pub fn data_mut(&mut self) -> &mut glm::Vec3 {
        &mut self.0
    }

    /// adds a data vector
    pub fn add(&mut self, vec: glm::Vec3) {
        self.0 += vec;
    }

    /// creates a new position filled with zeros (origin)
    pub fn zeros() -> Self {
        Position(glm::Vec3::zeros())
    }
}

impl Default for Position {
    fn default() -> Self {
        Position::zeros()
    }
}

/// velocity in 3D space
#[derive(Debug, Clone, PartialEq)]
pub struct Velocity(glm::Vec3);

impl Velocity {
    /// creates a new velocity
    pub const fn new(dx: f32, dy: f32, dz: f32) -> Self {
        Velocity(glm::Vec3::new(dx, dy, dz))
    }

    /// yields a copy of the stored data
    pub fn data_clone(&self) -> glm::Vec3 {
        self.0.clone()
    }

    /// grants immutable access to the stored data
    pub fn data(&self) -> &glm::Vec3 {
        &self.0
    }

    /// grants mutable access to the stored data
    pub fn data_mut(&mut self) -> &mut glm::Vec3 {
        &mut self.0
    }

    /// adds a data vector
    pub fn add(&mut self, vec: glm::Vec3) {
        self.0 += vec;
    }

    /// creates a new velocity filled with zeros
    pub fn zeros() -> Self {
        Velocity(glm::Vec3::zeros())
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Velocity::zeros()
    }
}

/// acceleration in 3D space
#[derive(Debug, Clone, PartialEq)]
pub struct Acceleration(glm::Vec3);

impl Acceleration {
    /// creates a new acceleration
    pub const fn new(ddx: f32, ddy: f32, ddz: f32) -> Self {
        Acceleration(glm::Vec3::new(ddx, ddy, ddz))
    }

    /// yields a copy of the stored data
    pub fn data_clone(&self) -> glm::Vec3 {
        self.0.clone()
    }

    /// grants immutable access to the stored data
    pub fn data(&self) -> &glm::Vec3 {
        &self.0
    }

    /// grants mutable access to the stored data
    pub fn data_mut(&mut self) -> &mut glm::Vec3 {
        &mut self.0
    }

    /// adds a data vector
    pub fn add(&mut self, vec: glm::Vec3) {
        self.0 += vec;
    }

    /// creates a new acceleration filled with zeros
    pub fn zeros() -> Self {
        Acceleration(glm::Vec3::zeros())
    }
}

impl Default for Acceleration {
    fn default() -> Self {
        Acceleration::zeros()
    }
}

/// binds all of the motion-specific data together
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MotionState {
    pub velocity: Velocity,
    pub acceleration: Acceleration,
}

/// efficient 32bit color representation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Default)]
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
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Default, Hash, Eq)]
pub enum MeshType {
    #[default]
    Cube,
    Plane,
    Sphere,
}

/// wether or not a mesh is colored or textured
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub enum MeshAttribute {
    Colored(Color32),
    Textured(GLuint),
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

impl Default for MeshAttribute {
    fn default() -> Self {
        Colored(Color32::default())
    }
}

/// component wrapper struct for `std::time::Instant` to track physics time
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct TouchTime(Instant);

impl TouchTime {
    /// wrapper for Instant::now()
    pub fn now() -> Self {
        TouchTime(Instant::now())
    }

    /// reset the internal time point to Instant::now()
    pub fn reset(&mut self) {
        self.0 = Instant::now();
    }

    /// generate the delta time since the last reset in seconds
    pub fn delta_time_f32(&self) -> f32 {
        self.0.elapsed().as_secs_f32()
    }
}

/// stores all of the associated sound controller ids for an entity
#[derive(Debug, Clone, PartialEq)]
pub struct SoundControl {
    id: u64,
}
