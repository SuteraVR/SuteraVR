use futures::{
    task::{LocalFutureObj, LocalSpawn, LocalSpawnExt},
    Future,
};
use godot::{engine::Engine, prelude::*};
use tokio::{
    runtime::{Builder, Runtime},
    task::LocalSet,
};

#[derive(Default)]
struct SharedLocalPool {
    local_set: LocalSet,
}

impl LocalSpawn for SharedLocalPool {
    fn spawn_local_obj(
        &self,
        future: LocalFutureObj<'static, ()>,
    ) -> Result<(), futures::task::SpawnError> {
        godot_print!("SharedLocalPool Generated");
        self.local_set.spawn_local(future);

        Ok(())
    }
}

thread_local! {
    static EXECUTOR: &'static SharedLocalPool = {
        Box::leak(Box::default())
    };
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct AsyncExecutorDriver {
    runtime: Runtime,
    base: Base<Node>,
}

#[godot_api]
impl INode for AsyncExecutorDriver {
    fn init(base: Base<Node>) -> Self {
        godot_print!("AsyncExecutorDriver Initialized!");
        Self {
            base,
            runtime: Builder::new_multi_thread()
                .worker_threads(4)
                .enable_io()
                .enable_time()
                .build()
                .unwrap(),
        }
    }
    fn process(&mut self, _delta: f64) {
        // EXECUTOR.with(|e| {
        //     self.runtime
        //         .block_on(async {
        //             e.local_set
        //                 .run_until(async { tokio::task::spawn_local(async {}).await })
        //                 .await
        //         })
        //         .unwrap()
        // })
    }
}

impl AsyncExecutorDriver {
    pub fn spawn(&self, f: impl Future<Output = ()> + Send + 'static) {
        // self.inner.spawn_ok(TokioIo { handle, inner: f });
        // EXECUTOR.with(|e| e.spawn_local(f)).unwrap();
        self.runtime.spawn(f);
    }
}
pub fn driver() -> Gd<AsyncExecutorDriver> {
    Engine::singleton()
        .get_singleton(StringName::from("AsyncExecutorDriver"))
        .unwrap()
        .cast::<AsyncExecutorDriver>()
}
