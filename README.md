<div align="center">
    <img src="assets/images/icon.png" width="300" height="300" alt="Leaf" />
</div>

> [!Note]
> This project is neither finished nor stable. There are architectual quirks that make it unergonomic to use in some respects. I have not worked on this project in a serious way for quite some time for a variety of reasons and will not work on it any more in the future. More on the why on my [website](https://luca-jt.github.io/articles/rust-game-programming/). I figured, that this version is still worth making public, as I invested a lot of time into it and it might be interesting.

___

So far the engine provides the following features out of the box:
- A simple archetypal ECS for efficient entity data storage
- Simple entity data manipulation with a data base-like query system
- A fully automated 3D and 2D rendering system based on entity data with various effects
- An event system with dynamically dispatched Listeners and function events
- Physics automatically simulated based on entity data
- 3D mesh manipulation algorithms such as LODs
- OS events are already managed and accessable via the event system
- 3D-audio with sound effects attachable to entities

# Usage
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
- The ``on_frame_update`` function runs once every frame. You can use it to implement your app logic. This includes changing the engine's internal state and running your own code.

If you want to use the internal logger to get information about what is happening under the hood, you can set the ``LOG_LVL`` environment variable to one of ``log``'s logging levels (``error``, ``warn``, ``info``, ``debug``, ``trace``). Setting the variable to ``trace`` enables all log messages but will cause a significant performance hit.

## Examples
- All examples are located in the `/examples` folder
- Clone the repository
- Run them with:
```sh
# runs the "3D" example
cargo run --release --example 3D
```

### Credits
This library uses [fyrox-sound](https://github.com/FyroxEngine/Fyrox/tree/master/fyrox-sound) for audio file decoding and 3D audio composing. Its functionality is integrated in the engine's audio system to interact with the entity data.
