use crate::internal_prelude::*;
use fyrox_sound::pool::Handle;
use fyrox_sound::source::SoundSource;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// The trait that all components need to implement. Must be manually implemented.
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
            #[doc = "Creates a new "]
            #[doc = stringify!($component)]
            #[doc = " for given input values."]
            pub const fn new(x: f32, y: f32, z: f32) -> Self {
                Self(Vec3::new(x, y, z))
            }

            /// Grants immutable access to the stored data.
            pub fn data(&self) -> &Vec3 {
                &self.0
            }

            /// Grants mutable access to the stored data.
            pub fn data_mut(&mut self) -> &mut Vec3 {
                &mut self.0
            }
        }

        impl From<Vec3> for $component {
            fn from(value: Vec3) -> Self {
                Self(value)
            }
        }
    };
}

/// Wrapper for an object scaling. Each vector component is the factor for each dimension.
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct Scale(Vec3);

impl Component for Scale {}

impl_basic_vec_ops!(Scale);

impl Scale {
    /// Creates an even scaling with a given factor.
    pub const fn from_factor(factor: f32) -> Self {
        Self::new(factor, factor, factor)
    }

    /// Calculates the scale matrix from the stored scalars.
    pub fn scale_matrix(&self) -> Mat4 {
        glm::scale(&Mat4::identity(), &self.0)
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }
}

/// Used for object orientation in 3D space. Based on a quaternion.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Orientation(pub Quat);

impl Component for Orientation {}

impl Orientation {
    /// Creates a new ``Orientation`` with angle in degrees around an axis.
    pub fn new(angle: f32, axis: Vec3) -> Self {
        Self(glm::quat_angle_axis(angle.to_radians(), &axis))
    }

    /// Generates the rotation matrix for the stored quaternion.
    pub fn rotation_matrix(&self) -> Mat4 {
        glm::quat_to_mat4(&self.0)
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Self::new(0.0, X_AXIS)
    }
}

/// Position in 3D space.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Position(Vec3);

impl Component for Position {}

impl_basic_vec_ops!(Position);

impl Position {
    /// Creates a new position at the coordinate origin.
    pub const fn origin() -> Self {
        Self(Vec3::new(0.0, 0.0, 0.0))
    }
}

impl_arithmetic_basics!(Position);

impl Default for Position {
    fn default() -> Self {
        Self::origin()
    }
}

/// Velocity in 3D space. Enables some physics system effects.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Velocity(Vec3);

impl Component for Velocity {}

impl_basic_vec_ops!(Velocity);

impl Velocity {
    /// Creates a new ``Velocity`` filled with zeros.
    pub const fn zero() -> Self {
        Self(Vec3::new(0.0, 0.0, 0.0))
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

/// Acceleration in 3D space.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Acceleration(Vec3);

impl Component for Acceleration {}

impl_basic_vec_ops!(Acceleration);

impl Acceleration {
    /// Creates a new ``Acceleration`` filled with zeros.
    pub const fn zero() -> Self {
        Self(Vec3::new(0.0, 0.0, 0.0))
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

/// Describes an angular momentum by the rotational axis (rhs rotation) and its length (momentum).
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct AngularMomentum(Vec3);

impl Component for AngularMomentum {}

impl_basic_vec_ops!(AngularMomentum);

impl AngularMomentum {
    /// Creates a new ``AngularMomentum`` filled with zeros.
    pub const fn zero() -> Self {
        Self(Vec3::new(0.0, 0.0, 0.0))
    }

    /// Creates a new ``AngularMomentum`` from a given axis.
    pub fn from_axis(axis: Vec3) -> Self {
        Self(axis)
    }
}

impl_arithmetic_basics!(AngularMomentum);

impl Default for AngularMomentum {
    fn default() -> Self {
        Self::zero()
    }
}

/// Contains all of the data for a renderable object in 3D.
#[derive(Debug, Clone)]
pub struct Renderable {
    pub mesh_type: MeshType,
    pub mesh_attribute: MeshAttribute,
    pub material_source: MaterialSource,
    pub shader_type: ShaderType,
}

impl Component for Renderable {}

/// Enables gravity physics and is used for all computations involving forces.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RigidBody {
    pub(crate) density: f32,
    pub(crate) inv_inertia_tensor: Mat3,
    pub(crate) center_of_mass: Vec3,
    pub(crate) mass: f32,
    pub(crate) friction: f32,
    pub(crate) restitution: f32,
}

impl Component for RigidBody {}

impl RigidBody {
    /// Changes the density of the rigid body (should be > 0).
    pub fn with_density(mut self, density: f32) -> Self {
        self.density = density;
        self
    }

    /// Changes the friction of the rigid body (should be >= 0).
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    /// Changes the restitution coefficient of the rigid body (clamped to [0, 1]).
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            density: 1.0,
            inv_inertia_tensor: Mat3::identity(),
            center_of_mass: ORIGIN,
            mass: 1.0,
            friction: 0.5,
            restitution: 0.0,
        }
    }
}

/// Stores all of the associated sound handles for an entity.
#[derive(Debug, Clone)]
pub struct SoundController {
    pub handles: Vec<Handle<SoundSource>>,
    pub(crate) doppler_pitch: f64,
    pub(crate) last_pos: Vec3,
}

impl Component for SoundController {}

impl SoundController {
    /// Creates a new default ``SoundController`` component with no handles.
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
            doppler_pitch: 1.0,
            last_pos: ORIGIN,
        }
    }

    /// Creates a new ``SoundController`` with given handles attached.
    pub fn from_handles(handles: &[Handle<SoundSource>]) -> Self {
        let mut handle_vec = Vec::new();
        handle_vec.extend_from_slice(handles);

        Self {
            handles: handle_vec,
            doppler_pitch: 1.0,
            last_pos: ORIGIN,
        }
    }
}

/// Adds a hitbox to an entity and specifies the position and scale of it relative to the enity's (requires ``Renderable`` to work and should only be used with meshes that have a volume).
#[derive(Debug, Copy, Clone)]
pub struct Collider {
    pub hitbox_type: HitboxType,
    pub offset: Vec3,
    pub scale: Scale,
}

impl Component for Collider {}

impl Collider {
    /// Creates a new ``Collider`` from a given hitbox type.
    pub fn from_type(hitbox_type: HitboxType) -> Self {
        Self {
            hitbox_type,
            offset: ORIGIN,
            scale: Scale::default(),
        }
    }
}

/// Point light component (requires a ``Position`` to work).
#[derive(Debug, Copy, Clone)]
pub struct PointLight {
    pub color: Color32,
    pub intensity: GLfloat,
    pub has_shadows: bool,
}

impl Component for PointLight {}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: Color32::WHITE,
            intensity: 1.0,
            has_shadows: true,
        }
    }
}

/// Directional light component (requires a ``Position`` to work).
#[derive(Debug, Copy, Clone)]
pub struct DirectionalLight {
    pub color: Color32,
    pub intensity: GLfloat,
    pub direction: Vec3,
}

impl Component for DirectionalLight {}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            color: Color32::WHITE,
            intensity: 1.0,
            direction: -Y_AXIS,
        }
    }
}

/// 64bit flag bitmap for enabling special entity behavior (default: all turned off, the same as component not present).
/// ### Info
/// You can use this component independantly of the rest of the engine if you want to.
/// The bits 6-63 do not influence engine behavior and are free to customize.
#[derive(Debug, Default)]
pub struct EntityFlags(u64);

impl Component for EntityFlags {}

impl EntityFlags {
    /// Creates a new ``EntityFlags`` component with the given flags already set.
    pub fn from_flags(flags: &[u64]) -> Self {
        let mut instance = Self::default();
        for flag in flags {
            instance.set_bit(*flag, true);
        }
        instance
    }

    /// Gets the bool value of the ``n``'th flag bit (``n`` is in ``(0..=63)``) (bit constants available in ``constants::bits``).
    pub fn get_bit(&self, n: u64) -> bool {
        ((self.0 >> n) & 1) == 1
    }

    /// Sets the bool value of the ``n``'th flag bit (``n`` is in ``(0..=63)``) (bit constants available in ``constants::bits``).
    pub fn set_bit(&mut self, n: u64, value: bool) {
        self.0 = (self.0 & !(1 << n)) | ((value as u64) << n);
    }
}

/// Defines the level of detail for a mesh if used in combination with a ``MeshType``.
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

/// Holds data for sprite rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct Sprite {
    pub source: SpriteSource,
    pub position: SpritePosition,
    pub layer: SpriteLayer,
}

impl Component for Sprite {}

/// Data structures that are not internally useful as a sole component but might have purpose in relation to other components. Many of them might also be usable as general-purpose types.
pub mod utils {
    use crate::internal_prelude::*;
    use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

    /// Efficient 32bit color representation.
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

        /// Uses the format ``0xRRGGBBAA``.
        pub const fn from_hex(hex: u32) -> Self {
            Self::from_rgba(
                ((hex & 0xFF000000) >> 24) as u8,
                ((hex & 0x00FF0000) >> 16) as u8,
                ((hex & 0x0000FF00) >> 8) as u8,
                (hex & 0x000000FF) as u8,
            )
        }

        /// Creates a new ``Color32`` from float RGB values in range [0, 1].
        pub const fn from_float_rgb(r: f32, g: f32, b: f32) -> Self {
            Self::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
        }

        /// Creates a new ``Color32`` from float RGBA values in range [0, 1].
        pub const fn from_float_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
            Self::from_rgba(
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                (a * 255.0) as u8,
            )
        }

        /// Converts the color to a float RGBA vector. This is most commonly used in rendering.
        pub fn to_vec4(&self) -> Vec4 {
            let r = self.r as f32 / 255.0;
            let g = self.g as f32 / 255.0;
            let b = self.b as f32 / 255.0;
            let a = self.a as f32 / 255.0;

            vec4(r, g, b, a)
        }
    }

    impl Default for Color32 {
        fn default() -> Self {
            Self::WHITE
        }
    }

    /// Determines what mesh should be attached to an entity.
    #[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Hash, Eq)]
    pub enum MeshType {
        Triangle,
        Plane,
        Cube,
        Custom(MeshHandle),
    }

    impl MeshType {
        /// Maps the ``MeshType`` to the respective ``MeshHandle``. This is e.g. useful to access built-in mesh data.
        pub fn mesh_handle(&self) -> MeshHandle {
            match self {
                Self::Triangle => 1,
                Self::Plane => 2,
                Self::Cube => 3,
                Self::Custom(handle) => *handle,
            }
        }
    }

    /// Determines wether or not a mesh is colored or textured.
    #[derive(Debug, PartialEq, Clone)]
    pub enum MeshAttribute {
        Colored(Color32),
        Textured(Texture),
    }

    impl MeshAttribute {
        /// Returns the color if present.
        pub fn color(&self) -> Option<Color32> {
            match self {
                Self::Textured(_) => None,
                Self::Colored(color) => Some(*color),
            }
        }
        /// Returns the texture if present.
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

    /// Represents a material that influences the rendering of the entity. You can either specify custom parameters or inherit the material data from the ``.obj`` file of the ``MeshType`` of the entity.
    #[derive(Debug, PartialEq, Clone)]
    pub enum MaterialSource {
        Custom(Material),
        Named(String),
        Inherit,
    }

    impl Default for MaterialSource {
        fn default() -> Self {
            Self::Custom(Material::default())
        }
    }

    /// Specific material data with components either being a value or a texture to sample from.
    #[derive(Debug, PartialEq, Clone)]
    pub struct Material {
        pub ambient: Ambient,
        pub diffuse: Diffuse,
        pub specular: Specular,
        pub shininess: Shininess,
        pub normal_texture: Option<String>,
    }

    impl Material {
        /// convert a loaded ``.mtl`` file
        pub(crate) fn from_mtl(mtl: &tobj::Material) -> Self {
            Self {
                ambient: mtl.ambient.map_or(
                    mtl.ambient_texture
                        .clone()
                        .map_or_else(Ambient::default, |path| Ambient::Texture(path)),
                    |color| Ambient::Value(Color32::from_float_rgb(color[0], color[1], color[2])),
                ),
                diffuse: mtl.diffuse.map_or(
                    mtl.diffuse_texture
                        .clone()
                        .map_or_else(Diffuse::default, |path| Diffuse::Texture(path)),
                    |color| Diffuse::Value(Color32::from_float_rgb(color[0], color[1], color[2])),
                ),
                specular: mtl.specular.map_or(
                    mtl.specular_texture
                        .clone()
                        .map_or_else(Specular::default, |path| Specular::Texture(path)),
                    |color| Specular::Value(Color32::from_float_rgb(color[0], color[1], color[2])),
                ),
                shininess: mtl.shininess.map_or(
                    mtl.shininess_texture
                        .clone()
                        .map_or_else(Shininess::default, |path| Shininess::Texture(path)),
                    |value| Shininess::Value(value),
                ),
                normal_texture: mtl.normal_texture.clone(),
            }
        }

        /// returns the ambient color as a rgb float vec if present
        pub(crate) fn ambient_color_val(&self) -> Option<Vec3> {
            match self.ambient {
                Ambient::Value(color) => Some(color.to_vec4().xyz()),
                Ambient::Texture(_) => None,
            }
        }

        /// returns the name of the used ambient texture if present
        pub(crate) fn ambient_texture(&self) -> Option<&str> {
            match &self.ambient {
                Ambient::Value(_) => None,
                Ambient::Texture(name) => Some(name.as_str()),
            }
        }

        /// returns the diffuse color as a rgb float vec if present
        pub(crate) fn diffuse_color_val(&self) -> Option<Vec3> {
            match self.diffuse {
                Diffuse::Value(color) => Some(color.to_vec4().xyz()),
                Diffuse::Texture(_) => None,
            }
        }

        /// returns the name of the used diffuse texture if present
        pub(crate) fn diffuse_texture(&self) -> Option<&str> {
            match &self.diffuse {
                Diffuse::Value(_) => None,
                Diffuse::Texture(name) => Some(name.as_str()),
            }
        }

        /// returns the specular color as a rgb float vec if present
        pub(crate) fn specular_color_val(&self) -> Option<Vec3> {
            match self.specular {
                Specular::Value(color) => Some(color.to_vec4().xyz()),
                Specular::Texture(_) => None,
            }
        }

        /// returns the name of the used specular texture if present
        pub(crate) fn specular_texture(&self) -> Option<&str> {
            match &self.specular {
                Specular::Value(_) => None,
                Specular::Texture(name) => Some(name.as_str()),
            }
        }

        /// returns the shininess as a float if present
        pub(crate) fn shininess_val(&self) -> Option<f32> {
            match self.shininess {
                Shininess::Value(val) => Some(val),
                Shininess::Texture(_) => None,
            }
        }

        /// returns the name of the used shininess texture if present
        pub(crate) fn shininess_texture(&self) -> Option<&str> {
            match &self.shininess {
                Shininess::Value(_) => None,
                Shininess::Texture(name) => Some(name.as_str()),
            }
        }
    }

    impl Default for Material {
        fn default() -> Self {
            Self {
                ambient: Ambient::default(),
                diffuse: Diffuse::default(),
                specular: Specular::default(),
                shininess: Shininess::default(),
                normal_texture: None,
            }
        }
    }

    /// Ambient color, stores either a color value or the texture name.
    #[derive(Debug, PartialEq, Clone)]
    pub enum Ambient {
        Value(Color32),
        Texture(String),
    }

    impl Default for Ambient {
        fn default() -> Self {
            Self::Value(Color32::WHITE)
        }
    }

    /// Diffuse color, stores either a color value or the texture name.
    #[derive(Debug, PartialEq, Clone)]
    pub enum Diffuse {
        Value(Color32),
        Texture(String),
    }

    impl Default for Diffuse {
        fn default() -> Self {
            Self::Value(Color32::WHITE)
        }
    }

    /// Specular color, stores either a color value or the texture name.
    #[derive(Debug, PartialEq, Clone)]
    pub enum Specular {
        Value(Color32),
        Texture(String),
    }

    impl Default for Specular {
        fn default() -> Self {
            Self::Value(Color32::WHITE)
        }
    }

    /// Shininess (or glossiness), stores either a value or the texture name.
    #[derive(Debug, PartialEq, Clone)]
    pub enum Shininess {
        Value(f32),
        Texture(String),
    }

    impl Default for Shininess {
        fn default() -> Self {
            Self::Value(32.0)
        }
    }

    /// Component wrapper for `std::time::Instant` to track time.
    #[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
    pub struct TimePoint(Instant);

    impl TimePoint {
        /// Wrapper for ``Instant::now()``.
        pub fn now() -> Self {
            TimePoint(Instant::now())
        }

        /// Reset the internal time point to ``Instant::now()``.
        pub fn reset(&mut self) {
            self.0 = Instant::now();
        }

        /// Generate the delta time since the last reset in seconds.
        pub fn delta_time(&self) -> TimeDuration {
            TimeDuration(self.0.elapsed().as_secs_f32())
        }
    }

    /// Time duration unit in seconds used for physics computations.
    #[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
    pub struct TimeDuration(pub f32);

    impl_arithmetic_basics!(TimeDuration);

    impl Div<TimeDuration> for TimeDuration {
        type Output = f32;

        fn div(self, rhs: TimeDuration) -> Self::Output {
            self.0 / rhs.0
        }
    }

    /// Represents a texture that can be used in rendering. Specifies all loading parameters and the file location.
    #[derive(Debug, PartialOrd, PartialEq, Clone, Hash, Eq)]
    pub struct Texture {
        pub path: Rc<Path>,
        pub filtering: Filtering,
        pub wrapping: Wrapping,
        pub color_space: ColorSpace,
        pub is_transparent: bool,
    }

    impl Texture {
        /// Creates a texture component with default attributes from a source path.
        pub fn default_from_path(path: Rc<Path>) -> Self {
            Self {
                path,
                filtering: Filtering::default(),
                wrapping: Wrapping::default(),
                color_space: ColorSpace::RGBA8,
                is_transparent: false,
            }
        }
    }

    /// Texture filtering option for rendering.
    #[derive(Debug, PartialOrd, PartialEq, Copy, Clone, Hash, Eq, Default)]
    pub enum Filtering {
        #[default]
        Linear,
        Nearest,
    }

    /// Texture wrapping mode.
    #[derive(Debug, PartialOrd, PartialEq, Copy, Clone, Hash, Eq, Default)]
    pub enum Wrapping {
        #[default]
        Repeat,
        MirroredRepeat,
        ClampToEdge,
        ClampToBorder,
    }

    /// Defines the color space for a texture.
    #[derive(Debug, PartialOrd, PartialEq, Copy, Clone, Hash, Eq)]
    pub enum ColorSpace {
        SRGBA,
        RGBA8,
    }

    /// Hitbox type specifier for an entity.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum HitboxType {
        ConvexHull,
        SimplifiedConvexHull,
        Sphere,
        Box,
    }

    /// Defines on what depth layer the sprite will be rendered on (``Layer0`` is nearest).
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
        /// Converts the sprite layer to the corresponding z coordinate for rendering. Also used internally and mainly public for info.
        pub fn to_z_coord(self) -> f32 {
            map_range((0.0, 9.0), (0.8, -0.8), self as isize as f32)
        }
    }

    /// Defines ways to source sprite data from.
    #[derive(Debug, Clone, PartialEq)]
    pub enum SpriteSource {
        Sheet(SpriteSheetSource),
        Colored(Color32),
        Single(Rc<Path>),
    }

    /// Source data for a sprite from a sprite sheet.
    #[derive(Debug, Clone, PartialEq, Hash, Eq)]
    pub struct SpriteSheetSource {
        pub path: Rc<Path>,
        pub pixel_index: (usize, usize),
        pub pixel_size: (usize, usize),
    }

    /// Sprite position on a defined grid or in absolute values.
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum SpritePosition {
        Grid(Vec2),
        Absolute(Vec2),
    }

    /// all shader variants for entity rendering
    #[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, Default)]
    pub enum ShaderType {
        #[default]
        Basic,
        Passthrough,
        //Custom()
    }
}
