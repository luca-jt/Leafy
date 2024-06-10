pub struct App {
    settings: u32,
}

impl App {
    pub fn new() -> Self {
        Self { settings: 0 }
    }

    pub fn run(&mut self) {
        println!("Hamburger");
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.settings = 0;
    }
}
