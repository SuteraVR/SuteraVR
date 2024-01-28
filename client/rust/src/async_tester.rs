use std::time::Duration;

use godot::prelude::*;
use tokio::time::sleep;

use crate::{async_driver::tokio, set_logger_target};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct AsyncTimerForTokioTest {
    base: Base<Node>,
}

set_logger_target!("AsyncTimerForTokioTest");

#[godot_api]
impl INode for AsyncTimerForTokioTest {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
    fn ready(&mut self) {
        tokio().bind().spawn("timer", async move {
            loop {
                sleep(Duration::from_secs(3)).await;
                godot_print!("Hello from green thread!");
            }
        })
    }
}
