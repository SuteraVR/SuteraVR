use suteravr_lib::clocking::ClockingFramingError;
use thiserror::Error;
use tokio::sync::oneshot;

#[derive(Debug, Error)]
pub enum TcpServerError {
    #[error(transparent)]
    ConnectingError(std::io::Error),
    #[error(transparent)]
    ShutdownRecvError(#[from] oneshot::error::RecvError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    FramingError(#[from] ClockingFramingError),
}
