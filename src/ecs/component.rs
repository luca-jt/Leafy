use nalgebra_glm as glm;

pub type Position = glm::Vec3;

pub type Quaternion = glm::Vec4;

pub type Velocity = glm::Vec3;

pub type Acceleration = glm::Vec3;

/// binds all of the motion-specific data together
pub enum MotionState {
    Moving(Velocity, Acceleration),
    Fixed,
}

impl MotionState {
    /// produces a default motion state with all fields being zero
    pub fn zeros() -> MotionState {
        MotionState::Moving(Velocity::zeros(), Acceleration::zeros())
    }

    /// checks if the state is fixed
    pub fn is_fixed(&self) -> bool {
        match self {
            MotionState::Fixed => true,
            _ => false,
        }
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
