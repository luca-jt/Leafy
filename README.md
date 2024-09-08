# Falling Leaf
This project is a Mini-Engine designed to be a great starting point for building games in Rust using OpenGL.\
It is written in pure Rust and with minimal external dependecies.\

![icon](https://github.com/luca-jt/Falling-Leaf/assets/82292985/c87b1c7c-119f-4934-9eb2-0854884bc3f5)\

So far, the Falling Leaf Engine provides the following features out of the Box:
- A simple ECS (Entity Component System) for
- A fully automated Rendering System based on entity data
- 3D-Audio with Soundeffects attachable to entities
- A non-polling Eventsystem with dynamically dispatched Listeners and function events (expandable by the user)
- An immediate-mode UI library
- 2D and 3D rendering
- OS events are already managed and accessable via the event system
- A functional windowed app up and running in seconds

## Build Process
- add the following to your `Cargo.toml` file:
```
[dependencies]
falling_leaf = "*"
```
- simply run:
```
cargo run
```
## Overview
- create an app that implements the `FallingLeafApp` trait and run the app like this:
```rs
use fl_core::engine::Engine;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new();
    let mut engine = Engine::new();
    engine.run(app)
}
```
## Examples
- located in the `/examples` folder
