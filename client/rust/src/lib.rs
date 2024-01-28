pub mod async_driver;

use async_driver::AsyncExecutorDriver;
use godot::{
    engine::{notify::NodeNotification, Engine},
    prelude::*,
};
use tokio::sync::oneshot;

use crate::async_driver::driver;

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

#[derive(GodotClass)]
#[class(base=Node)]
struct TestAsyncTask {
    tx: Option<oneshot::Sender<()>>,
    counter: f64,
    base: Base<Node>,
}

#[godot_api]
impl INode for TestAsyncTask {
    fn init(base: Base<Node>) -> Self {
        Self {
            tx: None,
            base,
            counter: 0f64,
        }
    }

    fn ready(&mut self) {
        godot_print!("Spawning...");
        let (tx, rx) = oneshot::channel();
        self.tx = Some(tx);
        driver().bind().spawn(async move {
            godot_print!("Waiting...");
            rx.await.unwrap();
            godot_print!("Received!");
        });
    }

    fn process(&mut self, delta: f64) {
        if self.counter >= 0f64 {
            self.counter += delta;
        }
        if self.counter > 5f64 {
            let tx = self.tx.take().unwrap();
            driver().bind().spawn(async move {
                godot_print!("Sending...");
                tx.send(()).unwrap();
            });
            self.counter = f64::NEG_INFINITY;
        }
    }

    fn on_notification(&mut self, what: NodeNotification) {
        if let NodeNotification::WmCloseRequest | NodeNotification::ExitTree = what {
            godot_print!("Closing...");
            self.tx.take().unwrap().send(()).unwrap();
        }
    }
}
