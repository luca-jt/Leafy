# Falling Leaf
This project is designed to be a great starting point for building simple games in Rust using SDL and OpenGL.\
This is not meant to be a full on game engine with an actual GUI Editor, but a template that you can use to start building the game immediately.\
![icon](https://github.com/luca-jt/Falling-Leaf/assets/82292985/c87b1c7c-119f-4934-9eb2-0854884bc3f5)

## Build Process
- install vcpkg
- run the following commands:
```
cargo install cargo-vcpkg
cargo vcpkg build
cargo build
```
## Usage
Probably the most interesting part is the game state. There is an ``init`` function and an ``update`` function that you can customize to your specific needs. It is the heart and soul of the project and lets you add entities to the game world and alter their state.\
The rendering and physics is already taken care of.\
Another possible point of interest is the event system. This is where all of the keyboard input is handled.\
In general: feel free to change anything you like - this is a template after all.
