pub mod async_driver;
pub mod async_tester;
pub mod logger;
pub mod tcp;

use async_driver::AsyncExecutorDriver;
use godot::{engine::Engine, prelude::*};

struct SuteraClientLib;

#[gdextension]
unsafe impl ExtensionLibrary for SuteraClientLib {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            Engine::singleton().register_singleton(
                StringName::from("AsyncExecutorDriver"),
                AsyncExecutorDriver::new_alloc().upcast(),
            );
        }
    }
    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            Engine::singleton().unregister_singleton(StringName::from("AsyncExecutorDriver"));
        }
    }
}
