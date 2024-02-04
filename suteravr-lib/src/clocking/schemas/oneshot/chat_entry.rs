use alkahest::alkahest;
use chrono::{DateTime, Local};

use crate::messaging::id::PlayerId;

#[derive(Debug, Clone)]
pub struct ChatEntry {
    pub send_at: DateTime<Local>,
    pub sender: PlayerId,
    pub message: String,
}

#[derive(Debug)]
#[alkahest(Formula, Serialize, Deserialize)]
pub struct SendableChatEntry {
    pub send_at: String,
    pub sender: PlayerId,
    pub message: String,
}

impl From<ChatEntry> for SendableChatEntry {
    fn from(entry: ChatEntry) -> Self {
        Self {
            send_at: entry.send_at.to_rfc3339(),
            sender: entry.sender,
            message: entry.message,
        }
    }
}

impl From<SendableChatEntry> for ChatEntry {
    fn from(entry: SendableChatEntry) -> Self {
        Self {
            send_at: DateTime::parse_from_rfc3339(&entry.send_at)
                .unwrap()
                .with_timezone(&Local),
            sender: entry.sender,
            message: entry.message,
        }
    }
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
