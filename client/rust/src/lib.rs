use godot::prelude::*;
mod toolik;
struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
