pub trait Component {
    fn update(&mut self);
}

pub struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl Component for Position {
    fn update(&mut self) {
        todo!()
    }
}

pub struct Velocity {
    dx: f32,
    dy: f32,
    dz: f32,
}

impl Component for Velocity {
    fn update(&mut self) {
        todo!()
    }
}

pub struct Acceleration {
    ddx: f32,
    ddy: f32,
    ddz: f32,
}

impl Component for Acceleration {
    fn update(&mut self) {
        todo!()
    }
}
