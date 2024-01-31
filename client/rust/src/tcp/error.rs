use suteravr_lib::clocking::ClockingFramingError;
use thiserror::Error;
use tokio::sync::{mpsc::error::SendError, oneshot};

use super::requests::{Request, Response};

#[derive(Debug, Error)]
pub enum TcpServerError {
    #[error("The request cannot be sent.")]
    CannotSendRequest(SendError<Request>),
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
}
