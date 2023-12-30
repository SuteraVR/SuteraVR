use std::sync::Arc;
use tokio::sync::RwLock;

pub type Arw<T> = Arc<RwLock<T>>;
