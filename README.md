# Falling Leaf
This project is a Mini-Engine designed to be a great starting point for building games in Rust using OpenGL.\
It is written in pure Rust and with minimal external dependecies.

![icon](https://github.com/luca-jt/Falling-Leaf/assets/82292985/c87b1c7c-119f-4934-9eb2-0854884bc3f5)

So far the Falling Leaf Engine provides the following features out of the box:
- A simple ECS (Entity Component System) for efficient game data storage
- Simple entity data manipulation with a data base-like Query system
- A fully automated Rendering System based on entity data
- A non-polling Event System with dynamically dispatched Listeners and function events
- An immediate-mode UI library
- 2D and 3D rendering
- OS events are already managed and accessable via the event system
- A functional windowed app up and running in seconds
- 3D-Audio with sound effects attachable to entities

## Build Process
- add the following to your `Cargo.toml` file:
```
[dependencies]
falling_leaf = "0.1.0"
```
- simply run:
```
cargo run
```
## Overview
- create an app struct that implements the `FallingLeafApp` trait and run the app like this:
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
- all examples are located in the `/examples` folder
- run them with:
```
# runs the "test" example
cargo run --example test
```
### Credits
This library uses [fyrox-sound](https://github.com/FyroxEngine/Fyrox/tree/master/fyrox-sound) for audio file decoding and 3D audio composing. Its functionality is integrated in the engines' audio system to interact with the entity data.
