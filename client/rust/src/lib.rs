use godot::{engine::notify::NodeNotification, prelude::*};
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot;

struct SuteraClientLib;

#[gdextension]
unsafe impl ExtensionLibrary for SuteraClientLib {}
