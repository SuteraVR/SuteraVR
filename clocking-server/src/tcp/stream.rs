use std::net::SocketAddr;
use suteravr_lib::{
    clocking::{
        buffer::{ContentHeader, FrameBuffer},
        event_headers::EventRequest,
        traits::MessageAuthor,
        ClockingConnection, ClockingFrameUnit,
    },
    util::logger::EnvLogger,
    warn,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{
        mpsc::{self},
        oneshot,
    },
    task::{Builder, JoinHandle},
};

use crate::{errors::TcpServerError, shutdown::ShutdownReason, tcp::requests::OneshotRequest};

use super::requests::{Request, Response};

pub struct ClientMessageStream {
    shutdown_tx: oneshot::Sender<ShutdownReason>,

    // サーバー側が自発的にリクエストを生成することが今はあんまりない
    // そのうち実装します
    #[allow(dead_code)]
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
        let logger = EnvLogger {
            target: format!("stream {}", peer_addr),
        };

        let handle = Builder::new()
            .name(format!("Stream of {}", peer_addr).as_str())
            .spawn(async move {
                let connection = &mut connection;
                let mut shutdown = shutdown_rx;
                let mut frame_buffer = FrameBuffer::new(logger.clone());
                let receive = receive_tx;
                let mut send = send_rx;

                loop {
                    tokio::select! {
                        Some(response) = send.recv() => {
                            match response {
                                Response::Oneshot(oneshot) => {
                                    connection.write_frame(&ClockingFrameUnit::SuteraHeader(oneshot.sutera_header)).await?;
                                    connection.write_frame(&ClockingFrameUnit::SuteraStatus(oneshot.sutera_status)).await?;
                                    connection.write_frame(&ClockingFrameUnit::OneshotHeaders(oneshot.oneshot_header)).await?;
                                    connection.write_frame(&ClockingFrameUnit::Content(oneshot.payload)).await?;
                                },
                                Response::Event(event) => {
                                    connection.write_frame(&ClockingFrameUnit::SuteraHeader(event.sutera_header)).await?;
                                    connection.write_frame(&ClockingFrameUnit::SuteraStatus(event.sutera_status)).await?;
                                    connection.write_frame(&ClockingFrameUnit::EventHeader(event.event_header)).await?;
                                    connection.write_frame(&ClockingFrameUnit::Content(event.payload)).await?;
                                },
                            }
                        },
                        read = connection.read_frame() => {
                            match read {
                                Ok(Some(payload)) => {
                                    if let Some(received) = frame_buffer.append(payload, MessageAuthor::Client) {
                                        if received.sutera_status.is_some() {
                                            panic!("Received message contains sutera_header! (Maybe frame_buffer has bugs.)")
                                        }
                                        match received.content_header {
                                            ContentHeader::Oneshot(oneshot_header) => {
                                                receive.send(
                                                    Request::Oneshot(OneshotRequest::new(
                                                        received.sutera_header,
                                                        oneshot_header,
                                                        received.payload,
                                                        reply.clone()
                                                    ))
                                                ).await.map_err(TcpServerError::CannotSendRequest)?;
                                            },
                                            ContentHeader::Event(event_header) => {
                                                receive.send(
                                                    Request::Event(EventRequest::new(
                                                        received.sutera_header,
                                                        event_header,
                                                        received.payload
                                                    ))
                                                ).await.map_err(TcpServerError::CannotSendRequest)?;
                                            },

                                        }
                                    }
                                }
                                Ok(None) => {
                                    break;
                                }
                                Err(e)=> {
                                    warn!(logger, "{}", e);
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

        Ok((
            Self {
                shutdown_tx,
                receive_rx,
                send_tx,
            },
            handle,
        ))
    }

    #[inline]
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
