use std::net::SocketAddr;

use log::error;
use suteravr_lib::clocking::{traits::MessageAuthor, ClockingConnection, ClockingFrameUnit};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{mpsc, oneshot},
    task::{Builder, JoinHandle},
};

use crate::{errors::TcpServerError, shutdown::ShutdownReason};

use super::requests::{Request, Response};

pub struct ClientMessageStream {
    handle: JoinHandle<Result<(), TcpServerError>>,
    peer_addr: SocketAddr,
    shutdown_tx: oneshot::Sender<ShutdownReason>,
    receive_rx: mpsc::Receiver<Request>,
}

impl ClientMessageStream {
    pub fn new<W: AsyncReadExt + AsyncWriteExt + Unpin + Send + 'static>(
        stream: W,
        peer_addr: SocketAddr,
    ) -> Result<Self, TcpServerError> {
        let mut connection = ClockingConnection::new(stream, MessageAuthor::Client);
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<ShutdownReason>();
        let (receive_tx, receive_rx) = mpsc::channel::<Request>(32);

        let handle = Builder::new()
            .name(format!("Stream of {}", peer_addr).as_str())
            .spawn(async move {
                let connection = &mut connection;
                let mut shutdown = shutdown_rx;
                let receive = receive_tx;

                loop {
                    tokio::select! {
                        read = connection.read_frame() => {
                            match read {
                                Ok(Some(payload)) => {
                                }
                                Ok(None) => {
                                    break;
                                }
                                Err(e)=> {
                                    error!("{}: {}", peer_addr, e);
                                    break;
                                }
                            }
                        },
                        _shutdown = &mut shutdown => {
                            connection.shutdown_stream().await.map_err(TcpServerError::ShutdownError)?;
                            break;
                        },
                    }
                }
                Ok::<(), TcpServerError>(())
            })
            .map_err(TcpServerError::SpawnError)?;

        Ok(Self {
            handle,
            peer_addr,
            shutdown_tx,
            receive_rx,
        })
    }

    pub async fn shutdown_and_wait(self, reason: ShutdownReason) -> Result<(), TcpServerError> {
        self.shutdown_tx
            .send(reason)
            .map_err(|_| TcpServerError::ThreadDead)?;
        self.handle.await?
    }
}
