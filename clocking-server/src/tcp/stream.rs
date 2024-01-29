use std::net::SocketAddr;

use suteravr_lib::clocking::{traits::MessageAuthor, ClockingConnection};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::oneshot,
    task::Builder,
};

use crate::{errors::TcpServerError, shutdown::ShutdownReason};

pub struct ClientMessageStream<W: AsyncReadExt + AsyncWriteExt + Unpin> {
    connection: ClockingConnection<W>,
    peer_addr: SocketAddr,
    shutdown_tx: oneshot::Sender<ShutdownReason>,
}

impl<W: AsyncReadExt + AsyncWriteExt + Unpin> ClientMessageStream<W> {
    pub fn new(stream: W, peer_addr: SocketAddr) -> Result<Self, TcpServerError> {
        let connection = ClockingConnection::new(stream, MessageAuthor::Client);

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<ShutdownReason>();

        Builder::new()
            .name(format!("Stream of {}", peer_addr).as_str())
            .spawn(async move {
                let shutdown = shutdown_rx;
            })
            .map_err(|e| TcpServerError::SpawnError(e))?;

        Ok(Self {
            connection,
            peer_addr,
            shutdown_tx,
        })
    }
}
