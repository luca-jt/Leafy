<div align="center">
    <img src="assets/images/icon.png" width="300" height="300" alt="Leaf" />
    <h1>Falling Leaf</h1>
</div>

[![License (MIT)](https://img.shields.io/crates/l/falling_leaf)](https://github.com/luca-jt/Falling-Leaf/blob/master/LICENSE)
[![Dependency status](https://deps.rs/...)](https://deps.rs/...)
[![docs.rs](https://img.shields.io/badge/docs-website-blue)](https://docs.rs/...)
[![Lines of code](https://tokei.rs/...)](https://github.com/luca-jt/Falling-Leaf)

___
This project is a 3D and 2D Mini-Engine designed to be a great starting point for building games in Rust.\
It is written in pure Rust and with minimal external dependecies.
___

So far the Falling Leaf Engine provides the following features out of the box:
- A simple ECS (Entity Component System) for efficient game data storage
- Simple entity data manipulation with a data base-like Query system
- A fully automated Rendering System based on entity data with various effects
- A non-polling Event System with dynamically dispatched Listeners and function events
- An immediate-mode UI library
- 3D and 2D rendering
- Physics automatically simulated based on entity data (3D + 2D)
- Mesh manipulation algorithms such as LODs
- OS events are already managed and accessable via the event system
- A functional windowed app up and running in seconds
- 3D-Audio with sound effects attachable to entities
- ... and much more

## Usage
- Add the following to your `Cargo.toml` file:
```
[dependencies]
falling_leaf = "0.1.0"
```

## Examples
- All examples are located in the `/examples` folder
- Clone the repository
- Run them with:
```sh
# runs the "3D" example
cargo run --release --example 3D
```

## Usage
- Create an app struct that implements the `FallingLeafApp` trait and run the app like this:
```rs
use falling_leaf::engine_builder::EngineAttributes;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new();
    let mut engine = EngineAttributes::new().build_engine().unwrap();
    engine.run(app)
}
```
- The ``init`` function runs one time at engine start-up. It is supposed to be used to run the setup for your app. You can specify settings for different systems and run your own setup code for your app.
- The ``on_frame_update`` function runs once every frame. You can use it to implement your app logic. This includes changing the engines' internal state, adding UI features, and running your own code.

This crate exports the crates ``glm``(nalgebra_glm), ``winit``, ``log``, and ``env_logger``. They can be used to perform math operations, logging, and use certain engine features. This way you don't have to manually add them yourself.\
If you want to use the internal logger to get information about what is happening under the hood, you can set the ``LOG_LVL`` environment variable to one of ``log``'s logging levels (``error``, ``warn``, ``info``, ``debug``, ``trace``). Setting the variable to ``trace`` enables all log messages but will cause a significant performance hit.

## Overview
The engine consists of different systems, that all serve their respective purposes and have their respective settings.
You can access them with their respective accessor functions. That way, you can also access the app that you decided to run the engine with. This is necessary e.g. when it comes to function events - more on that later.
The only functionality that the engine is directly responsible for is triggering engine-wide events that are handled by the event system. This is done for reasons of implementation.
In the following we will go over all of the different systems and their usage. This section provides additional information to the [Docs](https://docs.rs/...).

### Video System
The video system is responsible for creating the window controlling its parameters. It also grants the user of the engine access to those parameters.
This includes, but is not limited to:
- Specifics about the event loop behavior
- Enabling the built-in camera movement with the mouse
- Modifying window parameters

### Audio System
The audio system is responsible for managing the context of the audio engine.
It controls

### Event System
...

### Rendering System
...

### Animation System
...

### Entity Manager
- always use custom structs for components
...

### UI Library
...

## Unsafe
The only ``unsafe`` code segments in this crate are the OpenGL calls and some implementation details in the mutable queries.\
In the future the unsafeness of mutable entity manager query calls will be broadly resolved. The only reason the call itself is unsafe at the moment is that otherwhise there would be a borrowing problem with the entity manager. The disconnect between the whole manager and the internal ECS is not trivial and not just solved by introducing a RefCell or something similar and are linked to lifetime issues associated with that.

### Credits
This library uses [fyrox-sound](https://github.com/FyroxEngine/Fyrox/tree/master/fyrox-sound) for audio file decoding and 3D audio composing. Its functionality is integrated in the engines' audio system to interact with the entity data.
