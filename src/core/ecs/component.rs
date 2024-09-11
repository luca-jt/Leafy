use crate::glm;
use crate::systems::audio_system::SoundHandleID;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};
use std::time::Instant;
use MeshAttribute::*;

/// wrapper struct for an object scaling
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct Scale(glm::Vec3);

impl Scale {
    /// create a new scale for given input values
    pub fn new(x_scale: f32, y_scale: f32, z_scale: f32) -> Self {
        Self(glm::Vec3::new(x_scale, y_scale, z_scale))
    }

    /// grants immutable access to the stored data
    pub fn data(&self) -> &glm::Vec3 {
        &self.0
    }

    /// grants mutable access to the stored data
    pub fn data_mut(&mut self) -> &mut glm::Vec3 {
        &mut self.0
    }

    /// creates an even scaling with a given factor
    pub fn from_factor(factor: f32) -> Self {
        Self::new(factor, factor, factor)
    }

    /// calculates the scale matrix for the stored scalars
    pub fn scale_matrix(&self) -> glm::Mat4 {
        glm::scale(&glm::Mat4::identity(), &self.0)
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }
}

/// used for object orientation in 3D space
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Orientation {
    pub angle: f32,
    pub axis: glm::Vec3,
}

impl Orientation {
    /// creates a new orientation with angle in degrees around axis (x, y, z)
    pub fn new(angle: f32, x: f32, y: f32, z: f32) -> Self {
        Self {
            angle,
            axis: glm::Vec3::new(x, y, z),
        }
    }

    /// generates the rotation matrix for the stored quaternion
    pub fn rotation_matrix(&self) -> glm::Mat4 {
        glm::quat_to_mat4(&glm::quat_angle_axis(self.angle.to_radians(), &self.axis))
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Self::new(0.0, 1.0, 0.0, 0.0)
    }
}

/// position in 3D space
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Position(glm::Vec3);

impl Position {
    /// creates a new position
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self(glm::Vec3::new(x, y, z))
    }

    /// grants immutable access to the stored data
    pub fn data(&self) -> &glm::Vec3 {
        &self.0
    }

    /// grants mutable access to the stored data
    pub fn data_mut(&mut self) -> &mut glm::Vec3 {
        &mut self.0
    }

    /// creates a new position filled with zeros (origin)
    pub fn zeros() -> Self {
        Self(glm::Vec3::zeros())
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        Position(self.0 + rhs.0)
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Position) {
        self.0 += rhs.0;
    }
}

impl Sub for Position {
    type Output = Position;

    fn sub(self, rhs: Position) -> Self::Output {
        Position(self.0 - rhs.0)
    }
}

impl SubAssign for Position {
    fn sub_assign(&mut self, rhs: Position) {
        self.0 -= rhs.0;
    }
}

impl Mul<f32> for Position {
    type Output = Position;

    fn mul(self, rhs: f32) -> Self::Output {
        Position(self.0 * rhs)
    }
}

impl MulAssign<f32> for Position {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}

impl Default for Position {
    fn default() -> Self {
        Position::zeros()
    }
}

/// velocity in 3D space
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Velocity(glm::Vec3);

impl Velocity {
    /// creates a new velocity
    pub const fn new(dx: f32, dy: f32, dz: f32) -> Self {
        Self(glm::Vec3::new(dx, dy, dz))
    }

    /// grants immutable access to the stored data
    pub fn data(&self) -> &glm::Vec3 {
        &self.0
    }

    /// grants mutable access to the stored data
    pub fn data_mut(&mut self) -> &mut glm::Vec3 {
        &mut self.0
    }

    /// creates a new velocity filled with zeros
    pub fn zeros() -> Self {
        Self(glm::Vec3::zeros())
    }
}

impl Add for Velocity {
    type Output = Velocity;

    fn add(self, rhs: Velocity) -> Self::Output {
        Velocity(self.0 + rhs.0)
    }
}

impl AddAssign for Velocity {
    fn add_assign(&mut self, rhs: Velocity) {
        self.0 += rhs.0;
    }
}

impl Sub for Velocity {
    type Output = Velocity;

    fn sub(self, rhs: Velocity) -> Self::Output {
        Velocity(self.0 - rhs.0)
    }
}

impl SubAssign for Velocity {
    fn sub_assign(&mut self, rhs: Velocity) {
        self.0 -= rhs.0;
    }
}

impl Mul<TimeDuration> for Velocity {
    type Output = Position;

    fn mul(self, rhs: TimeDuration) -> Self::Output {
        Position(self.0 * rhs.0)
    }
}

impl Mul<f32> for Velocity {
    type Output = Velocity;

    fn mul(self, rhs: f32) -> Self::Output {
        Velocity(self.0 * rhs)
    }
}

impl MulAssign<f32> for Velocity {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Velocity::zeros()
    }
}

/// acceleration in 3D space
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Acceleration(glm::Vec3);

impl Acceleration {
    /// creates a new acceleration
    pub const fn new(ddx: f32, ddy: f32, ddz: f32) -> Self {
        Self(glm::Vec3::new(ddx, ddy, ddz))
    }

    /// grants immutable access to the stored data
    pub fn data(&self) -> &glm::Vec3 {
        &self.0
    }

    /// grants mutable access to the stored data
    pub fn data_mut(&mut self) -> &mut glm::Vec3 {
        &mut self.0
    }

    /// creates a new acceleration filled with zeros
    pub fn zeros() -> Self {
        Self(glm::Vec3::zeros())
    }
}

impl Add for Acceleration {
    type Output = Acceleration;

    fn add(self, rhs: Acceleration) -> Self::Output {
        Acceleration(self.0 + rhs.0)
    }
}

impl AddAssign for Acceleration {
    fn add_assign(&mut self, rhs: Acceleration) {
        self.0 += rhs.0;
    }
}

impl Sub for Acceleration {
    type Output = Acceleration;

    fn sub(self, rhs: Acceleration) -> Self::Output {
        Acceleration(self.0 - rhs.0)
    }
}

impl SubAssign for Acceleration {
    fn sub_assign(&mut self, rhs: Acceleration) {
        self.0 -= rhs.0;
    }
}

impl Mul<TimeDuration> for Acceleration {
    type Output = Velocity;

    fn mul(self, rhs: TimeDuration) -> Self::Output {
        Velocity(self.0 * rhs.0)
    }
}

impl Mul<f32> for Acceleration {
    type Output = Acceleration;

    fn mul(self, rhs: f32) -> Self::Output {
        Acceleration(self.0 * rhs)
    }
}

impl MulAssign<f32> for Acceleration {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}

impl Default for Acceleration {
    fn default() -> Self {
        Acceleration::zeros()
    }
}

/// efficient 32bit color representation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct Color32([u8; 4]);

impl Color32 {
    pub const WHITE: Self = Self::from_rgb(255, 255, 255);
    pub const BLACK: Self = Self::from_rgb(0, 0, 0);
    pub const TRANSPARENT: Self = Self([0, 0, 0, 0]);
    pub const RED: Self = Self::from_rgb(255, 0, 0);
    pub const GREEN: Self = Self::from_rgb(0, 255, 0);
    pub const BLUE: Self = Self::from_rgb(0, 0, 255);
    pub const YELLOW: Self = Self::from_rgb(255, 255, 0);
    pub const CYAN: Self = Self::from_rgb(0, 255, 255);
    pub const PURPLE: Self = Self::from_rgb(255, 0, 255);

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

impl Default for Color32 {
    fn default() -> Self {
        Self::WHITE
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
    Textured(&'static str),
}

impl MeshAttribute {
    /// returns the color if present
    pub fn color(&self) -> Option<Color32> {
        match self {
            Textured(_) => None,
            Colored(color) => Some(*color),
        }
    }
    /// returns the texture path if present
    pub fn texture_path(&self) -> Option<&str> {
        match self {
            Textured(path) => Some(*path),
            Colored(_) => None,
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
    pub fn delta_time(&self) -> TimeDuration {
        TimeDuration(self.0.elapsed().as_secs_f32())
    }
}

/// time duration unit for physics computations
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct TimeDuration(pub f32);

impl Mul<f32> for TimeDuration {
    type Output = TimeDuration;

    fn mul(self, rhs: f32) -> Self::Output {
        TimeDuration(self.0 * rhs)
    }
}

/// stores all of the associated sound controller ids for an entity
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundController {
    pub(crate) id: SoundHandleID,
}

/// responsible for entity collision checking
#[derive(Debug, Clone, PartialEq)]
pub enum Hitbox {
    Quad {
        position: glm::Vec3,
        orientation: Orientation,
        width: f32,
        height: f32,
        link: Box<Hitbox>,
    },
    Cube {
        position: glm::Vec3,
        orientation: Orientation,
        width: f32,
        height: f32,
        depth: f32,
        link: Box<Hitbox>,
    },
    Sphere {
        position: glm::Vec3,
        radius: f32,
        link: Box<Hitbox>,
    },
    None,
}
/*
impl Hitbox {
    /// checks wether or not a hitbox is touching another hitbox
    pub fn hit_by(&self, other: &Hitbox) -> bool {
        match self {
            Hitbox::Quad(h1) => match other {
                Hitbox::Quad(h2) => true || self.hit_by(h2) || h1.hit_by(h2),
                Hitbox::Cube(h2) => true || self.hit_by(h2) || h1.hit_by(h2),
                Hitbox::Sphere(h2) => true || self.hit_by(h2) || h1.hit_by(h2),
                Hitbox::None => false,
            },
            Hitbox::Cube(h1) => match other {
                Hitbox::Quad(h2) => true || self.hit_by(h2) || h1.hit_by(h2),
                Hitbox::Cube(h2) => true || self.hit_by(h2) || h1.hit_by(h2),
                Hitbox::Sphere(h2) => true || self.hit_by(h2) || h1.hit_by(h2),
                Hitbox::None => false,
            },
            Hitbox::Sphere(h1) => match other {
                Hitbox::Quad(h2) => true || self.hit_by(h2) || h1.hit_by(h2),
                Hitbox::Cube(h2) => true || self.hit_by(h2) || h1.hit_by(h2),
                Hitbox::Sphere(h2) => true || self.hit_by(h2) || h1.hit_by(h2),
                Hitbox::None => false,
            },
            Hitbox::None => false,
        }
    }
}
*/
