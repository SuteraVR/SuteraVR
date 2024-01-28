use futures::Future;
use godot::{engine::Engine, prelude::*};
use tokio::runtime::{Builder, Runtime};

use crate::{log, set_logger_target};

set_logger_target!("AsyncExecutorDriver");

#[derive(GodotClass)]
#[class(base=Node)]
pub struct AsyncExecutorDriver {
    runtime: Runtime,
    base: Base<Node>,
}

#[godot_api]
impl INode for AsyncExecutorDriver {
    fn init(base: Base<Node>) -> Self {
        log!("Runtime initialized.");
        Self {
            base,
            runtime: Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .unwrap(),
        }
    }
}

impl AsyncExecutorDriver {
    /// Spawns a new task on the runtime.
    pub fn spawn(&self, name: &str, f: impl Future<Output = ()> + Send + 'static) {
        log!("Spawning task: {}", name);
        self.runtime.spawn(f);
    }
}

/// Returns the singleton instance of the AsyncExecutorDriver.
///
/// ```no_run
/// use sutera_client_lib::async_driver::tokio;
///
/// tokio().bind().spawn("my_task", async {
///     //...
/// });
/// ```
pub fn tokio() -> Gd<AsyncExecutorDriver> {
    Engine::singleton()
        .get_singleton(StringName::from("AsyncExecutorDriver"))
        .unwrap()
        .cast::<AsyncExecutorDriver>()
}
