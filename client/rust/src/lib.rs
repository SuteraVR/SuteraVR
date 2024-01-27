use godot::{engine::notify::NodeNotification, prelude::*};
use tokio::sync::oneshot;

struct SuteraClientLib;

#[gdextension]
unsafe impl ExtensionLibrary for SuteraClientLib {}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct TestNode2 {
    pub base: Base<Node>,
    pub a: Option<i32>,
}

#[godot_api]
impl INode for TestNode2 {
    fn init(base: Base<Self::Base>) -> Self {
        Self { base, a: None }
    }
    fn ready(&mut self) {
        godot_print!("Hello from Rust! Ready2!");
        godot_print!("{:?}", self.a);
    }
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct TestNode {
    tx: Option<oneshot::Sender<()>>,
    pub base: Base<Node>,
}

#[godot_api]
impl INode for TestNode {
    fn init(base: Base<Self::Base>) -> Self {
        Self { base, tx: None }
    }
    fn ready(&mut self) {
        godot_print!("Hello from Rust! Ready!");
        let (tx, rx) = oneshot::channel::<()>();
        tokio::spawn(async move {
            godot_print!("Thread here!");
            rx.await.unwrap();
            godot_print!("Hello from Rust! Done!");
        });
        self.tx = Some(tx);
    }
    fn on_notification(&mut self, what: NodeNotification) {
        match what {
            NodeNotification::ExitTree | NodeNotification::WmCloseRequest => {
                self.tx.take().unwrap().send(()).unwrap();
                godot_print!("Hello from Rust! Exit!");
            }
            _ => {}
        }
    }
}
