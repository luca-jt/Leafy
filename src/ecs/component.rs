use crate::glm;
use crate::utils::constants::*;
use fyrox_sound::pool::Handle;
use fyrox_sound::source::SoundSource;
use gl::types::GLfloat;
use std::any::Any;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::path::Path;
use std::rc::Rc;
use utils::*;

/// the trait that all components need to implement
pub trait Component: Any {}

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

        impl From<glm::Vec3> for $component {
            fn from(value: glm::Vec3) -> Self {
                Self(value)
            }
        }
    };
}

/// wrapper struct for an object scaling
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct Scale(glm::Vec3);

impl Component for Scale {}

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

impl Component for Orientation {}

impl Orientation {
    /// creates a new orientation with angle in degrees around axis
    pub fn new(angle: f32, axis: glm::Vec3) -> Self {
        Self(glm::quat_angle_axis(angle.to_radians(), &axis))
    }

    /// generates the rotation matrix for the stored quaternion
    pub fn rotation_matrix(&self) -> glm::Mat4 {
        glm::quat_to_mat4(&self.0)
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Self::new(0.0, X_AXIS)
    }
}

/// position in 3D space
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Position(glm::Vec3);

impl Component for Position {}

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
        Self::origin()
    }
}

/// velocity in 3D space, enables physics system effects
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Velocity(glm::Vec3);

impl Component for Velocity {}

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
        Self::zero()
    }
}

/// acceleration in 3D space
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Acceleration(glm::Vec3);

impl Component for Acceleration {}

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
        Self::zero()
    }
}

/// describes an angular momentum by the rotational axis (rhs rotation) and its length (momentum)
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct AngularMomentum(glm::Vec3);

impl Component for AngularMomentum {}

impl_basic_vec_ops!(AngularMomentum);

impl AngularMomentum {
    /// creates a new angular momentum filled with zeros
    pub const fn zero() -> Self {
        Self(glm::Vec3::new(0.0, 0.0, 0.0))
    }

    /// creates a new angular momentum from a given axis
    pub fn from_axis(axis: glm::Vec3) -> Self {
        Self(axis)
    }
}

impl_arithmetic_basics!(AngularMomentum);

impl Default for AngularMomentum {
    fn default() -> Self {
        Self::zero()
    }
}

/// contains all of the data for a renderable object in 3D
#[derive(Debug, Clone)]
pub struct Renderable {
    pub mesh_type: MeshType,
    pub mesh_attribute: MeshAttribute,
    pub material: Material,
}

impl Component for Renderable {}

impl Renderable {
    /// loads a renderable from a .obj file and associated .mtl files
    pub fn from_file(file: Rc<Path>) -> Self {
        todo!();
        Self {
            mesh_type: MeshType::Custom(file),
            mesh_attribute: MeshAttribute::default(),
            material: Material::default(),
        }
    }
}

/// enables gravity physics and is used for all computations involving forces
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RigidBody {
    pub(crate) density: f32,
    pub(crate) inv_inertia_tensor: glm::Mat3,
    pub(crate) center_of_mass: glm::Vec3,
    pub(crate) mass: f32,
    pub(crate) friction: f32,
    pub(crate) restitution: f32,
}

impl Component for RigidBody {}

impl RigidBody {
    /// changes the density of the rigid body (should be > 0)
    pub fn with_density(mut self, density: f32) -> Self {
        self.density = density;
        self
    }

    /// changes the friction of the rigid body (should be >= 0)
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    /// changes the restitution coefficient of the rigid body (clamped to [0, 1])
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            density: 1.0,
            inv_inertia_tensor: glm::Mat3::identity(),
            center_of_mass: ORIGIN,
            mass: 1.0,
            friction: 0.5,
            restitution: 0.0,
        }
    }
}

/// stores all of the associated sound handles for an entity
#[derive(Debug, Clone)]
pub struct SoundController {
    pub handles: Vec<Handle<SoundSource>>,
    pub(crate) doppler_pitch: f64,
    pub(crate) last_pos: glm::Vec3,
}

impl Component for SoundController {}

impl SoundController {
    /// creates a new default sound controller component with no handles
    pub fn new() -> Self {
        Self {
            handles: vec![],
            doppler_pitch: 1.0,
            last_pos: ORIGIN,
        }
    }

    /// creates a new sound controller with handles attached
    pub fn from_handles(handles: &[Handle<SoundSource>]) -> Self {
        Self {
            handles: handles.to_vec(),
            doppler_pitch: 1.0,
            last_pos: ORIGIN,
        }
    }
}

/// adds a hitbox to an entity and specifies the positional offset and scale of it relative to the enity's
/// (requires ``Renderable`` to work and should only be used with meshes that have a volume)
#[derive(Debug, Clone)]
pub struct Collider {
    pub(crate) hitbox_type: HitboxType,
    pub(crate) offset: glm::Vec3,
    pub(crate) scale: Scale,
    pub(crate) last_collisions: Vec<CollisionInfo>,
}

impl Component for Collider {}

impl Collider {
    /// creates a new collider from a given hitbox type
    pub fn new(hitbox_type: HitboxType) -> Self {
        Self {
            hitbox_type,
            offset: ORIGIN,
            scale: Scale::default(),
            last_collisions: vec![],
        }
    }

    /// sets the offset of the hitbox
    pub fn with_offset(mut self, offset: glm::Vec3) -> Self {
        self.offset = offset;
        self
    }

    /// sets the scale of the hitbox
    pub fn with_scale(mut self, scale: Scale) -> Self {
        self.scale = scale;
        self
    }

    /// access to the collision data of the collider from the last iteration
    pub fn collision_info(&self) -> &[CollisionInfo] {
        &self.last_collisions
    }
}

/// marks an entity as a point light source for the rendering system (requires a ``Position`` to work)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    pub color: Color32,
    pub intensity: GLfloat,
    pub direction: glm::Vec3,
}

impl Component for PointLight {}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: Color32::WHITE,
            intensity: 1.0,
            direction: -Y_AXIS,
        }
    }
}

/// 64bit flag bitmap for enabling special entity behavior (default: all turned off, the same as component not present)
/// ### Info
/// You can use this component independantly of the rest of the engine if you want to.
/// The bits 6-63 do not influence engine behavior and are free to customize.
#[derive(Debug, Default)]
pub struct EntityFlags(u64);

impl Component for EntityFlags {}

impl EntityFlags {
    /// creates a new ``EntityFlags`` component with the given flags already set
    pub fn from_flags(flags: &[u64]) -> Self {
        let mut instance = Self::default();
        for flag in flags {
            instance.set_bit(*flag, true);
        }
        instance
    }

    /// get the bool value of the ``n'th`` flag bit (``n`` is in ``(0..=63)``)
    /// (bit constants available in ``constants::bits``)
    pub fn get_bit(&self, n: u64) -> bool {
        ((self.0 >> n) & 1) == 1
    }

    /// set the bool value of the ``n'th`` flag bit (``n`` is in ``(0..=63)``)
    /// (bit constants available in ``constants::bits``)
    pub fn set_bit(&mut self, n: u64, value: bool) {
        self.0 = (self.0 & !(1 << n)) | ((value as u64) << n);
    }
}

/// sets the level of detail for a mesh if used in combination with a ``MeshType``
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Default)]
pub enum LOD {
    #[default]
    None = 0,
    LVL1,
    LVL2,
    LVL3,
    LVL4,
}

impl Component for LOD {}

/// holds data for sprite rendering
#[derive(Debug, Clone, PartialEq)]
pub struct Sprite {
    pub source: SpriteSource,
    pub position: SpritePosition,
    pub layer: SpriteLayer,
}

impl Component for Sprite {}

/// data structures that are not internally useful as a sole component but might have purpose in relation to other components
pub mod utils {
    use crate::glm;
    use crate::utils::tools::map_range;
    use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
    use std::path::Path;
    use std::rc::Rc;
    use std::time::Instant;

    /// efficient 32bit color representation
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
    pub struct Color32 {
        pub r: u8,
        pub g: u8,
        pub b: u8,
        pub a: u8,
    }

    impl Color32 {
        pub const WHITE: Self = Self::from_rgb(255, 255, 255);
        pub const GREY: Self = Self::from_rgb(128, 128, 128);
        pub const BLACK: Self = Self::from_rgb(0, 0, 0);
        pub const TRANSPARENT: Self = Self::from_rgba(0, 0, 0, 0);
        pub const RED: Self = Self::from_rgb(255, 0, 0);
        pub const GREEN: Self = Self::from_rgb(0, 255, 0);
        pub const BLUE: Self = Self::from_rgb(0, 0, 255);
        pub const YELLOW: Self = Self::from_rgb(255, 255, 0);
        pub const CYAN: Self = Self::from_rgb(0, 255, 255);
        pub const PURPLE: Self = Self::from_rgb(255, 0, 255);

        pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
            Self { r, g, b, a: 255 }
        }

        pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
            Self { r, g, b, a }
        }

        /// converts to a float rgba vector
        pub fn to_vec4(&self) -> glm::Vec4 {
            let r = self.r as f32 / 255.0;
            let g = self.g as f32 / 255.0;
            let b = self.b as f32 / 255.0;
            let a = self.a as f32 / 255.0;

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
        Torus,
        Custom(Rc<Path>),
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
                Self::Textured(_) => None,
                Self::Colored(color) => Some(*color),
            }
        }
        /// returns the texture path if present
        pub fn texture(&self) -> Option<&Texture> {
            match self {
                Self::Textured(texture) => Some(texture),
                Self::Colored(_) => None,
            }
        }
    }

    impl Default for MeshAttribute {
        fn default() -> Self {
            Self::Colored(Color32::default())
        }
    }

    /// represents a material that influences the rendering of the entity
    #[derive(Debug, PartialEq, Copy, Clone, Default)]
    pub struct Material {
        pub specular: f32,
        pub diffuse: f32,
        pub shininess: f32,
    }

    /// component wrapper struct for `std::time::Instant` to track time
    #[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
    pub struct TimePoint(Instant);

    impl TimePoint {
        /// wrapper for Instant::now()
        pub fn now() -> Self {
            TimePoint(Instant::now())
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

    /// represents all texture types with path data
    #[derive(Debug, PartialOrd, PartialEq, Clone, Hash, Eq)]
    pub struct Texture {
        pub path: Rc<Path>,
        pub filtering: Filtering,
        pub wrapping: Wrapping,
    }

    /// texture filtering option for rendering
    #[derive(Debug, PartialOrd, PartialEq, Clone, Hash, Eq)]
    pub enum Filtering {
        Linear,
        Nearest,
    }

    /// texture wrapping mode
    #[derive(Debug, PartialOrd, PartialEq, Clone, Hash, Eq)]
    pub enum Wrapping {
        Repeat,
        MirroredRepeat,
        ClampToEdge,
        ClampToBorder,
    }

    /// hitbox type specifier for an entity
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum HitboxType {
        ConvexHull,
        SimplifiedConvexHull,
        Sphere,
        Box,
    }

    /// stores info about the last frames' collisions in the ``Collider`` component
    #[derive(Debug, Copy, Clone)]
    pub struct CollisionInfo {
        pub momentum: glm::Vec3,
        pub point: glm::Vec3,
        pub normal: glm::Vec3,
    }

    /// defines on what depth layer the sprite will be rendered on (``Layer0`` is nearest)
    #[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
    pub enum SpriteLayer {
        Layer0 = 0,
        Layer1,
        Layer2,
        Layer3,
        Layer4,
        Layer5,
        Layer6,
        Layer7,
        Layer8,
        Layer9,
    }

    impl SpriteLayer {
        /// converts the sprite layer to the corresponding z coordinate for rendering
        pub fn to_z_coord(self) -> f32 {
            map_range((0.0, 9.0), (0.8, -0.8), self as isize as f32)
        }
    }

    /// defines ways to source sprite data from
    #[derive(Debug, Clone, PartialEq)]
    pub enum SpriteSource {
        Sheet(SpriteSheetSource),
        Colored(Color32),
        Single(Rc<Path>),
    }

    /// source data for a sprite from a sprite sheet
    #[derive(Debug, Clone, PartialEq, Hash, Eq)]
    pub struct SpriteSheetSource {
        pub path: Rc<Path>,
        pub pixel_index: (usize, usize),
        pub pixel_size: (usize, usize),
    }

    /// sprite position on a defined grid or in absolute values
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum SpritePosition {
        Grid(glm::Vec2),
        Absolute(glm::Vec2),
    }
}
