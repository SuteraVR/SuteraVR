//! Client-side Rust code
//!
//! [`suteravr-lib`][suteravr_lib]が使用できます。
//! ```no_run
//! use suteravr_lib::Foo;
//! ```
//!

use godot::prelude::*;
mod toolik;
struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
