use tokio::time;

use alkahest::alkahest;

use crate::messaging::id::PlayerId;

#[derive(Debug, Clone)]
pub struct ChatEntry {
    pub send_at: time::Instant,
    pub sender: PlayerId,
    pub message: String,
}

#[derive(Debug)]
#[alkahest(Formula, Serialize, Deserialize)]
pub struct SendChatMessageRequest {
    pub content: String,
}

#[derive(Debug)]
#[alkahest(Formula, Serialize, Deserialize)]
pub enum SendChatMessageResponse {
    Ok,
    Error,
}
