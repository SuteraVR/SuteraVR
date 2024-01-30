use std::{future::IntoFuture, net::SocketAddr};

use log::{debug, error, info, warn};
use suteravr_lib::clocking::{traits::MessageAuthor, ClockingConnection, ClockingFrameUnit};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{mpsc, oneshot},
    task::{Builder, JoinHandle},
};

use crate::{errors::TcpServerError, shutdown::ShutdownReason, tcp::requests::OneshotRequest};

use super::requests::{Request, Response};

struct FrameBuffer {
    pub buffer: Vec<ClockingFrameUnit>,
    peer_addr: SocketAddr,
}

impl FrameBuffer {
    #[inline]
    fn new(peer_addr: SocketAddr) -> Self {
        Self {
            buffer: Vec::with_capacity(4),
            peer_addr,
        }
    }

    #[inline]
    fn push(&mut self, unit: ClockingFrameUnit) {
        debug!("Frame from {}: {:?}", &self.peer_addr, unit);
        self.buffer.push(unit);
    }

    #[inline]
    fn len(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    fn clear(&mut self) {
        self.buffer.clear()
    }

    #[inline]
    fn get_into(&self, index: usize) -> Option<ClockingFrameUnit> {
        self.buffer.get(index).map(|v| v.clone())
    }
}

pub struct ClientMessageStream {
    peer_addr: SocketAddr,
    shutdown_tx: oneshot::Sender<ShutdownReason>,
    send_tx: mpsc::Sender<Response>,
    receive_rx: mpsc::Receiver<Request>,
}

impl ClientMessageStream {
    pub fn new<W: AsyncReadExt + AsyncWriteExt + Unpin + Send + 'static>(
        stream: W,
        peer_addr: SocketAddr,
    ) -> Result<(Self, JoinHandle<Result<(), TcpServerError>>), TcpServerError> {
        let mut connection = ClockingConnection::new(stream, MessageAuthor::Client);
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<ShutdownReason>();
        let (receive_tx, receive_rx) = mpsc::channel::<Request>(32);
        let (send_tx, send_rx) = mpsc::channel::<Response>(32);
        let reply = send_tx.clone();

        let handle = Builder::new()
            .name(format!("Stream of {}", peer_addr).as_str())
            .spawn(async move {
                let connection = &mut connection;
                let mut shutdown = shutdown_rx;
                let mut frame_buffer = FrameBuffer::new(peer_addr);
                
                let receive = receive_tx;
                let send = send_rx;

                
                let mut add_frame_buffer = |payload: ClockingFrameUnit| -> Option<Request> {
                    match payload {
                        ClockingFrameUnit::SuteraStatus(_) => {
                            error!("Unexpected SuteraStatus of ClockingConnection! ({})", peer_addr);
                            unreachable!();
                        }
                        ClockingFrameUnit::SuteraHeader(_) => {
                            let len = frame_buffer.len();
                            if frame_buffer.len() != 0 {
                                warn!("Frame from {}: Skipped {} frame(s).", peer_addr, len);
                                frame_buffer.clear();

                            }
                            frame_buffer.push(payload);
                        },
                        ClockingFrameUnit::Unfragmented(c) => {
                            warn!("Frame from {}: Receive {} unfragmented byte(s)", peer_addr, c.len());
                        },
                        ClockingFrameUnit::Content(payload) => {
                            let len = frame_buffer.len();
                            if len != 2 {
                                warn!("Frame from {}: Unexpected content, Skipped {} frame(s).", peer_addr, len);
                                frame_buffer.clear();
                                return None;
                            }
                            let Some(ClockingFrameUnit::SuteraHeader(sutera_header)) = frame_buffer.get_into(0) else {
                                return None;
                            };
                            match frame_buffer.get_into(1) {
                                Some(ClockingFrameUnit::OneshotHeaders(oneshot_header)) => {
                                    let request = OneshotRequest::new(
                                        sutera_header,
                                        oneshot_header,
                                        payload,
                                        reply.clone(),
                                    );
                                    info!("Request from {}: {:?}", peer_addr, &request);
                                    return Some(Request::Oneshot(request));
                                },
                                Some(_) | None => {
                                    return None;
                                },
                            }
                        }
                        _ => {
                            frame_buffer.push(payload);
                        }
                    }
                    None
                };

                loop {
                    tokio::select! {
                        read = connection.read_frame() => {
                            match read {
                                Ok(Some(payload)) => {
                                    if let Some(request) = add_frame_buffer(payload) {
                                        receive.send(request).await.map_err(TcpServerError::CannotSendRequest)?;
                                    }
                                }
                                Ok(None) => {
                                    break;
                                }
                                Err(e)=> {
                                    warn!("{}: {}", peer_addr, e);
                                    break;
                                }
                            }
                        },
                        _shutdown = &mut shutdown => {
                            connection.shutdown_stream().await.map_err(TcpServerError::ShutdownError)?;
                        },
                    }
                }
                Ok::<(), TcpServerError>(())
            })
            .map_err(TcpServerError::SpawnError)?;

        Ok((Self {
            peer_addr,
            shutdown_tx,
            receive_rx,
            send_tx,
        },handle))
    }

    pub async fn recv(&mut self) -> Option<Request> {
        self.receive_rx.recv().await
    }

    pub async fn shutdown(self, reason: ShutdownReason) -> Result<(), TcpServerError> {
        self.shutdown_tx
            .send(reason)
            .map_err(|_| TcpServerError::ThreadDead)?;
        Ok(())
    }
}
