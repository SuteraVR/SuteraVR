use std::time;

use crate::messaging::id::PlayerId;

#[derive(Debug, Clone)]
pub struct ChatEntry {
    pub send_at: time::Instant,
    pub sender: PlayerId,
    pub message: String,
}
