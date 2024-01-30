use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

use crate::tcp::requests::{Request, Response};

#[derive(Debug, Error)]
#[error(transparent)]
pub enum ClockingServerError {
    JoinError(#[from] tokio::task::JoinError),
    SpawnError(std::io::Error),
    IoError(#[from] std::io::Error),

    TcpServerError(#[from] TcpServerError),
}

#[derive(Debug, Error)]
pub enum TcpServerError {
    #[error("The thread was already dead.")]
    ThreadDead,
    #[error("The request cannot be sent.")]
    CannotSendRequest(SendError<Request>),
    #[error("The response cannot be sent.")]
    CannotSendResponse(SendError<Response>),
    #[error(transparent)]
    AcceptError(std::io::Error),
    #[error(transparent)]
    ShutdownError(std::io::Error),
    #[error(transparent)]
    ListenerBindError(std::io::Error),
    #[error(transparent)]
    SpawnError(std::io::Error),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
}
