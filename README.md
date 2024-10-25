<div align="center">
    <img src="assets/images/icon.png" width="300" height="300" alt="Leaf" />
    <h1>Falling Leaf</h1>
</div>

[![License (MIT)](https://img.shields.io/crates/l/falling_leaf)](https://github.com/luca-jt/Falling-Leaf/blob/master/LICENSE)
[![Dependency status](https://deps.rs/...)](https://deps.rs/...)
[![docs.rs](https://img.shields.io/badge/docs-website-blue)](https://docs.rs/...)
[![Lines of code](https://tokei.rs/...)](https://github.com/luca-jt/Falling-Leaf)

___
This project is a 3D and 2D Mini-Engine designed to be a great starting point for building games in Rust using OpenGL.\
It is written in pure Rust and with minimal external dependecies.
___

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

## Usage
- add the following to your `Cargo.toml` file:
```
[dependencies]
falling_leaf = "0.1.0"
```

## Examples
- all examples are located in the `/examples` folder
- clone the repository
- run them with:
```sh
# runs the "3D" example
cargo run --release --example 3D
```

## Overview
- create an app struct that implements the `FallingLeafApp` trait and run the app like this:
```rs
use fl_core::engine_builder::EngineAttributes;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new();
    let mut engine = EngineAttributes::new().build_engine().unwrap();
    engine.run(app)
}
```

### Credits
This library uses [fyrox-sound](https://github.com/FyroxEngine/Fyrox/tree/master/fyrox-sound) for audio file decoding and 3D audio composing. Its functionality is integrated in the engines' audio system to interact with the entity data.
