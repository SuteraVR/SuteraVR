use thiserror::Error;

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
