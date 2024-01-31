use futures::Future;
use godot::{engine::Engine, prelude::*};
use suteravr_lib::info;
use tokio::runtime::{Builder, Runtime};

use crate::logger::GodotLogger;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct AsyncExecutorDriver {
    runtime: Runtime,
    logger: GodotLogger,
    base: Base<Node>,
}

#[godot_api]
impl INode for AsyncExecutorDriver {
    fn init(base: Base<Node>) -> Self {
        let logger = GodotLogger {
            target: "AsyncDriver".to_string(),
        };
        info!(logger, "Runtime initialized.");
        Self {
            base,
            logger,
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
        info!(self.logger, "Spawning task: {}", name);
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
