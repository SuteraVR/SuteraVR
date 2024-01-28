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
#[error(transparent)]
pub enum TcpServerError {
    ListenerBindError(std::io::Error),
    SpawnError(std::io::Error),
    JoinError(#[from] tokio::task::JoinError),
}
