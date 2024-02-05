use suteravr_lib::clocking::ClockingFramingError;
use thiserror::Error;
use tokio::sync::{mpsc::error::SendError, oneshot};

use crate::{
    instance::{manager::InstancesControl, InstanceControl, PlayerControl},
    tcp::requests::{Request, Response},
};

#[derive(Debug, Error)]
#[error(transparent)]
pub enum ClockingServerError {
    JoinError(#[from] tokio::task::JoinError),
    SpawnError(std::io::Error),
    IoError(#[from] std::io::Error),

    TcpServerError(#[from] TcpServerError),
    InstanceError(#[from] InstanceError),

    CannotSendShutdown(anyhow::Error),

    #[error("The oneshot reply cannot be sent.")]
    CannotSendReply,
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
    FramingError(#[from] ClockingFramingError),
    #[error(transparent)]
    AcceptError(std::io::Error),
    #[error(transparent)]
    ShutdownError(std::io::Error),
    #[error(transparent)]
    FuseError(oneshot::error::RecvError),
    #[error(transparent)]
    ListenerBindError(std::io::Error),
    #[error(transparent)]
    SpawnError(std::io::Error),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    CannotSendToInstanceManager(#[from] SendError<InstancesControl>),
    #[error(transparent)]
    CannotSendToInstance(#[from] SendError<InstanceControl>),
    #[error(transparent)]
    CannotReceiveFromInstanceManager(oneshot::error::RecvError),
}

#[derive(Debug, Error)]
pub enum InstanceError {
    #[error(transparent)]
    CannotSendToPlayer(#[from] SendError<PlayerControl>),
    #[error(transparent)]
    SpawnError(std::io::Error),
}
