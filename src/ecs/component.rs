use crate::ecs::entity::EntityID;
use crate::glm;
use crate::systems::audio_system::SoundHandleID;
use crate::utils::constants::{X_AXIS, Y_AXIS};
use gl::types::GLfloat;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::path::{Path, PathBuf};
use std::time::Instant;
use MeshAttribute::*;

macro_rules! impl_arithmetic_basics {
    ($component:ident) => {
        impl Add for $component {
            type Output = $component;

            fn add(self, rhs: $component) -> Self::Output {
                $component(self.0 + rhs.0)
            }
        }

        impl AddAssign for $component {
            fn add_assign(&mut self, rhs: $component) {
                self.0 += rhs.0;
            }
        }

        impl Sub for $component {
            type Output = $component;

            fn sub(self, rhs: $component) -> Self::Output {
                $component(self.0 - rhs.0)
            }
        }

        impl SubAssign for $component {
            fn sub_assign(&mut self, rhs: $component) {
                self.0 -= rhs.0;
            }
        }

        impl Mul<f32> for $component {
            type Output = $component;

            fn mul(self, rhs: f32) -> Self::Output {
                $component(self.0 * rhs)
            }
        }

        impl MulAssign<f32> for $component {
            fn mul_assign(&mut self, rhs: f32) {
                self.0 *= rhs;
            }
        }

        impl Div<f32> for $component {
            type Output = $component;

            fn div(self, rhs: f32) -> Self::Output {
                $component(self.0 / rhs)
            }
        }

        impl DivAssign<f32> for $component {
            fn div_assign(&mut self, rhs: f32) {
                self.0 /= rhs;
            }
        }
    };
}

macro_rules! impl_basic_vec_ops {
    ($component:ident) => {
        impl $component {
            #[doc = "creates a new "]
            #[doc = stringify!($component)]
            #[doc = " for given input values"]
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
        }
    };
}

/// wrapper struct for an object scaling
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct Scale(glm::Vec3);

impl_basic_vec_ops!(Scale);

impl Scale {
    /// creates an even scaling with a given factor
    pub const fn from_factor(factor: f32) -> Self {
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
pub struct Orientation(pub glm::Quat);

impl Orientation {
    /// creates a new orientation with angle in degrees around axis
    pub fn new(angle: f32, axis: &glm::Vec3) -> Self {
        Self(glm::quat_angle_axis(angle.to_radians(), axis))
    }

    /// generates the rotation matrix for the stored quaternion
    pub fn rotation_matrix(&self) -> glm::Mat4 {
        glm::quat_to_mat4(&self.0)
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Self::new(0.0, &X_AXIS)
    }
}

/// position in 3D space
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Position(glm::Vec3);

impl_basic_vec_ops!(Position);

impl Position {
    /// creates a new position at the coordinate origin
    pub const fn origin() -> Self {
        Self(glm::Vec3::new(0.0, 0.0, 0.0))
    }
}

impl_arithmetic_basics!(Position);

impl Default for Position {
    fn default() -> Self {
        Position::origin()
    }
}

/// velocity in 3D space, enables physics system effects
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Velocity(glm::Vec3);

impl_basic_vec_ops!(Velocity);

impl Velocity {
    /// creates a new velocity filled with zeros
    pub const fn zero() -> Self {
        Self(glm::Vec3::new(0.0, 0.0, 0.0))
    }
}

impl_arithmetic_basics!(Velocity);

impl Mul<TimeDuration> for Velocity {
    type Output = Position;

    fn mul(self, rhs: TimeDuration) -> Self::Output {
        Position(self.0 * rhs.0)
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Velocity::zero()
    }
}

/// acceleration in 3D space
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Acceleration(glm::Vec3);

impl_basic_vec_ops!(Acceleration);

impl Acceleration {
    /// creates a new acceleration filled with zeros
    pub const fn zero() -> Self {
        Self(glm::Vec3::new(0.0, 0.0, 0.0))
    }
}

impl_arithmetic_basics!(Acceleration);

impl Mul<TimeDuration> for Acceleration {
    type Output = Velocity;

    fn mul(self, rhs: TimeDuration) -> Self::Output {
        Velocity(self.0 * rhs.0)
    }
}

impl Default for Acceleration {
    fn default() -> Self {
        Acceleration::zero()
    }
}

/// describes an angular velocity by the rotational axis (rhs rotaion) and its length (velocity in radiants per second)
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct AngularVelocity(pub glm::Vec3);

impl_basic_vec_ops!(AngularVelocity);

impl AngularVelocity {
    /// creates a new angular velocity filled with zeros
    pub const fn zero() -> Self {
        Self(glm::Vec3::new(0.0, 0.0, 0.0))
    }
}

impl_arithmetic_basics!(AngularVelocity);

impl Default for AngularVelocity {
    fn default() -> Self {
        AngularVelocity::zero()
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

        glm::vec4(r, g, b, a)
    }
}

impl Default for Color32 {
    fn default() -> Self {
        Self::WHITE
    }
}

/// all of the known mesh types
#[derive(Debug, Clone, PartialOrd, PartialEq, Hash, Eq)]
pub enum MeshType {
    Triangle,
    Plane,
    Cube,
    Sphere,
    Cylinder,
    Custom(PathBuf),
}

/// represents all texture types with path data
#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub enum Texture {
    Ice,
    Sand,
    Wall,
    Custom(PathBuf),
}

impl Texture {
    pub fn path(&self) -> &Path {
        match self {
            Texture::Ice => Path::new("ice.png"),
            Texture::Sand => Path::new("sand.png"),
            Texture::Wall => Path::new("wall.png"),
            Texture::Custom(path) => path.as_path(),
        }
    }
}

/// wether or not a mesh is colored or textured
#[derive(Debug, PartialEq, Clone)]
pub enum MeshAttribute {
    Colored(Color32),
    Textured(Texture),
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
    pub fn texture_path(&self) -> Option<&Path> {
        match self {
            Textured(texture) => Some(texture.path()),
            Colored(_) => None,
        }
    }
}

impl Default for MeshAttribute {
    fn default() -> Self {
        Colored(Color32::default())
    }
}

/// represents all material types with path data
#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub enum Material {
    Reflective,
    Custom(PathBuf),
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

/// time duration unit in seconds used for physics computations
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct TimeDuration(pub f32);

impl_arithmetic_basics!(TimeDuration);

impl Div<TimeDuration> for TimeDuration {
    type Output = f32;

    fn div(self, rhs: TimeDuration) -> Self::Output {
        self.0 / rhs.0
    }
}

/// enables gravity physics and is used for all computations involving forces
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Density(pub f32);

impl_arithmetic_basics!(Density);

impl Default for Density {
    fn default() -> Self {
        Density(1.0)
    }
}

/// contains an inertia tensor that is used for physics calculations
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MassDistribution(pub(crate) glm::Mat3);

impl Default for MassDistribution {
    fn default() -> Self {
        MassDistribution(glm::Mat3::identity())
    }
}

/// used for all computations involving friction
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Friction(pub f32);

impl_arithmetic_basics!(Friction);

/// force in 3D space
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Force(glm::Vec3);

impl_basic_vec_ops!(Force);

impl Force {
    /// creates a new force filled with zeros
    pub const fn zero() -> Self {
        Self(glm::Vec3::new(0.0, 0.0, 0.0))
    }
}

impl_arithmetic_basics!(Force);

/// stores all of the associated sound controller ids for an entity
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundController {
    pub(crate) id: SoundHandleID,
}

/// marker for an entity (enables collision physics)
#[derive(Debug, Clone, Copy)]
pub struct Hitbox;

/// marks an entity as a point light source for the rendering system, needs a position attached to work
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    pub color: Color32,
    pub intensity: GLfloat,
    pub direction: glm::Vec3,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: Color32::WHITE,
            intensity: 1.0,
            direction: -Y_AXIS,
        }
    }
}

/// identifier for a light source (used internally)
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LightSrcID(pub(crate) EntityID);

/// holds data for sprite rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sprite;
