use alkahest::DeserializeError;
use suteravr_lib::clocking::ClockingFramingError;
use thiserror::Error;
use tokio::sync::{mpsc::error::SendError, oneshot};

use super::requests::{Request, Response};

#[derive(Debug, Error)]
pub enum TcpServerError {
    #[error("ClockingConnection is not initialized or already closed.")]
    ConnectionNotFound,
    #[error("The request cannot be sent.")]
    CannotSendRequest(SendError<Request>),
    #[error("The oneshot reply cannot be sent.")]
    CannotSendOneshotReply,
    #[error("The response cannot be sent.")]
    CannotSendResponse(SendError<Response>),
    #[error(transparent)]
    ConnectingError(std::io::Error),
    #[error(transparent)]
    ShutdownRecvError(#[from] oneshot::error::RecvError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    FramingError(#[from] ClockingFramingError),
    #[error("Failed to deserialize the message.")]
    DeserializeError(DeserializeError),
}

impl From<DeserializeError> for TcpServerError {
    fn from(e: DeserializeError) -> Self {
        Self::DeserializeError(e)
    }
}
