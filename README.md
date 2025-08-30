<div align="center">
    <img src="assets/images/icon.png" width="300" height="300" alt="Leaf" />
    <h1>Leafy</h1>
</div>

[![License (MIT)](https://img.shields.io/crates/l/leafy)](https://github.com/luca-jt/Leafy/blob/master/LICENSE)
[![docs.rs](https://img.shields.io/badge/docs-website-blue)](https://docs.rs/...)
[![Lines of code](https://tokei.rs/...)](https://github.com/luca-jt/Leafy)

___
This project is a 3D and 2D Mini-Engine designed to be a great starting point for building games in Rust.
___

> [!Note]
> This project is not finished/stable. In fact, I do not recommed using it for anything serious. There are a lot of features that are not implemented and probably will never be implemented, and also some architectual quirks that make it unergonomic to use in some respects. I have not worked on this project in a serious way for quite some time now for a variety of reasons. More on that on my [website](https://luca-jt.github.io/projects/game-engine/). I am rewriting this project for private use in a different language. I am not sure if that will ever be published. I figured, that this version is still worth making public, as I invested a lot of time into it and it might be interesting for people. Over time this evolved more and more into a playground for me to explore new ideas rather than anything solid to be seriously used in game development.

So far the Leafy Engine provides the following features out of the box:
- A simple ECS (Entity Component System) for efficient game data storage
- Simple entity data manipulation with a data base-like Query system
- A fully automated Rendering System based on entity data with various effects
- A non-polling Event System with dynamically dispatched Listeners and function events
- An immediate-mode UI library
- 3D and 2D rendering
- Physics automatically simulated based on entity data
- 3D Mesh manipulation algorithms such as LODs
- OS events are already managed and accessable via the event system
- A functional windowed app up and running in seconds
- 3D-Audio with sound effects attachable to entities
- ... and much more

## Usage
- Add the following to your `Cargo.toml` file:
```
[dependencies]
leafy = "0.1.0"
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
- Create an app struct that implements the `LeafyApp` trait and run the app like this:
```rs
use leafy::engine_builder::EngineAttributes;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new();
    let mut engine = EngineAttributes::new().build_engine().unwrap();
    engine.run(app)
}
```
- The ``init`` function runs one time at engine start-up. It is supposed to be used to run the setup for your app. You can specify settings for different systems and run your own setup code for your app.
- The ``on_frame_update`` function runs once every frame. You can use it to implement your app logic. This includes changing the engines' internal state and running your own code.

This crate exports the crates ``glm``(nalgebra_glm), ``winit``, ``smallvec``, ``itertools``, ``log``, and ``env_logger``. They can be used to perform math operations, logging, and use certain engine features. This way you don't have to manually add them yourself.\
If you want to use the internal logger to get information about what is happening under the hood, you can set the ``LOG_LVL`` environment variable to one of ``log``'s logging levels (``error``, ``warn``, ``info``, ``debug``, ``trace``). Setting the variable to ``trace`` enables all log messages but will cause a significant performance hit.

## Overview
The engine consists of different systems, that all serve their respective purposes and have their respective settings.
You can access them with their respective accessor functions. That way, you can also access the app that you decided to run the engine with. This is necessary e.g. when it comes to function events - more on that later.
The only functionality that the engine is directly responsible for is triggering engine-wide events that are handled by the event system. This is done for implementation reasons.
In the following we will go over all of the different systems and their usage on a basic level. This section provides additional contextual information to the [Docs](https://docs.rs/...).
I would say that the source is quite easy to understand - so just check out the source directly if you need more specific info about features and implementation. The following outline might also give some clues about what to look for.

### Video System
The video system is responsible for creating the window controlling its parameters. It also grants the user of the engine access to those parameters.
This includes, but is not limited to:

- Specifics about the main loop behavior
- Frame rate modulation
- Enabling the built-in camera movement with the mouse
- Modifying window parameters
- Window and rendering viewport settings

All of this can be used to change the way the user interacts with the app and what he is allowed to do. The features are all quite self-explanatory.
Some features like the built-in camera movement that are not exclusive to one system are implemented as function events.

### Audio System
The audio system is responsible for managing the context of the audio engine.
You can load audio files and use the returned handle to controll the sound.
You can attach it to an entity using the ``SoundController`` component which will lead to the position of the sound being linked to the entity's position.
Depending on the movement of the entity and the bits set in the ``EntityFlags`` component, the sound's pitch will also automatically be updated for a **Doppler Effect**.
You can control the regular pitch independantly of this effect. You can use the regular stereo audio or use the built-in HRTF 3D audio. This also enables you to use certain other audio effects like reverb.
Depending on the animation speed fo the enigine, you can enable a variable pitch that is automatically applied to all sounds.
All effects and features that rely on 3D audio require the loaded sounds to be spatial in nature.
Deleted sound handles will automatically be removed from all components they are attached to.

### Event System
The event system is the main way of reacting to events form the operating system or the engine.
You can add listeners in the form of structs that implement the ``EventObserver<T>`` trait that are wrapped in shared references,
or use function events (modifiers) that have the signature ``fn(&T, &Engine<A>)`` where ``T`` is the event type and ``A`` the app that you use the engine with.
The latter is recommended.
You can also create your own events and use them for your program logic.
There are two types of built-in events:

- Read-only events (mainly from the operating system) that are not meant to be triggered manually
- Events that can also be triggered manually

The first type inlcudes almost all of the built-in events.
The built-in events that can be triggered manually are located in the ``user_space`` sub-module.

### Rendering System
All entities that have the necessary components for 3D or 2D rendering attached are automatically rendered.
For 2D sprites you can define sprite positions on defined grids or in an absolute way.
For 3D rendered entities, you can load them from ``.obj`` and ``.mtl`` files and manipulate them with e.g. scaling or materials/textures.
There are also a lot of other rendering-specific settings that can be modified here.
Some of them can be controlled not only by the presence of certain components, but also by various bits of ``EntityFlags``.

### Animation System
The animation system is responsible for all Physics simulations and animations for rendered entities.
The physics simulations are done dynamically, only require a small amount of base components and can be more specifically controlled by various bits of ``EntityFlags`` and other additional components.
The physics simulations include collision handling and mechanics. More specific information on the use cases of all the different components later.
There is also a broad spectrum of settings that can be directly accessed in the system struct.

### Entity Manager
The entity manager is responsible for storing all the data that is associated with entities.
This inlcudes the ECS that generically stores all of the components that are attached to entities. Assets can be managed here as well.
You can query the stored component data to implement generic entity behavior that is component-based. The respectice functions will return an Iterator over tuples of all the components.
You can also add filters to the queries to include/exclude entity types that contain certain components. Individual components can be queried mutably or immutably and also in an optional way, which yields a component only if it is present.
There are a lot of different built-in components that are used internally by the engine's systems, but you can also easily introduce your own components.
Components are arbitrary structs and enums.
For an overview of all the built-in components and their use cases, take a look at the Docs.

## Unsafe
The only ``unsafe`` code segments in this crate are the OpenGL calls and some implementation details in the mutable queries.\
In the future the unsafeness of mutable entity manager query calls will be broadly resolved. The only reason the call itself is unsafe at the moment is that otherwhise there would be a borrowing problem with the entity manager. The disconnect between the whole manager and the internal ECS is not trivial and not just solved by introducing a RefCell or something similar and are linked to lifetime issues associated with that.

### Credits
This library uses [fyrox-sound](https://github.com/FyroxEngine/Fyrox/tree/master/fyrox-sound) for audio file decoding and 3D audio composing. Its functionality is integrated in the engines' audio system to interact with the entity data.
