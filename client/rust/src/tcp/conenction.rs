use alkahest::deserialize;

use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};
use suteravr_lib::{
    clocking::{event_headers::EventTypes, schemas::oneshot::chat_entry::SendableChatEntry},
    error,
};

use godot::prelude::*;
use suteravr_lib::{
    clocking::{
        buffer::{ContentHeader, FrameBuffer},
        traits::MessageAuthor,
        ClockingConnection, ClockingFrameUnit,
    },
    info,
    messaging::id::MessageId,
    warn,
};
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_rustls::{
    rustls::{pki_types::ServerName, ClientConfig},
    TlsConnector,
};

use crate::{
    async_driver::tokio,
    logger::GodotLogger,
    signal_names::SIGNAL_NEW_TEXTCHAT_MESSAGE,
    tcp::{
        error::TcpServerError,
        requests::{EventMessage, OneshotRequest},
        ClockerConnection,
    },
};

use super::{
    requests::{Request, Response},
    ShutdownReason,
};

pub struct Connection {
    pub shutdown_tx: oneshot::Sender<ShutdownReason>,
    pub _receive_rx: mpsc::Receiver<Request>,
    pub send_tx: mpsc::Sender<Response>,
    pub handle: JoinHandle<()>,
}
impl Connection {
    pub fn new<T: Into<String> + Send + 'static>(
        logger: GodotLogger,
        instance_id: InstanceId,
        config: ClientConfig,
        name: T,
        addr: T,
    ) -> Self {
        info!(logger, "Making connection...");

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<ShutdownReason>();
        let (receive_tx, receive_rx) = mpsc::channel::<Request>(32);
        let (send_tx, mut send_rx) = mpsc::channel::<Response>(32);

        let server_logger = logger.clone();
        let _reply = send_tx.clone();

        let server = async move {
            let mut reply_senders = HashMap::<MessageId, oneshot::Sender<Request>>::new();
            let name = name.into();
            let addr = addr.into();
            let logger = server_logger;

            info!(logger, "Connecting to {}({}) ...", name, addr);

            let connector = TlsConnector::from(Arc::new(config));
            let dnsname = ServerName::try_from(name).unwrap();

            let stream = TcpStream::connect(&addr)
                .await
                .map_err(TcpServerError::ConnectingError)?;

            let stream = connector
                .connect(dnsname, stream)
                .await
                .map_err(TcpServerError::ConnectingError)?;

            info!(logger, "Connection established!");

            let mut connection = ClockingConnection::new(stream, MessageAuthor::Server);
            let mut frame_buffer = FrameBuffer::new(logger.clone());

            let receive = receive_tx;

            loop {
                tokio::select! {
                    Some(request) = send_rx.recv() => {
                        match request {
                            Response::Oneshot(oneshot) => {
                                connection.write_frame(&ClockingFrameUnit::SuteraHeader(oneshot.sutera_header)).await?;
                                connection.write_frame(&ClockingFrameUnit::OneshotHeaders(oneshot.oneshot_header)).await?;
                                connection.write_frame(&ClockingFrameUnit::Content(oneshot.payload)).await?;
                            },
                            Response::OneshotWithReply(oneshot, sender) => {
                                let Entry::Vacant(o) = reply_senders.entry(oneshot.oneshot_header.message_id) else {
                                    error!(logger, "MessageId {:?} is already occupied!", oneshot.oneshot_header.message_id);
                                    panic!();
                                };
                                o.insert(sender);
                                connection.write_frame(&ClockingFrameUnit::SuteraHeader(oneshot.sutera_header)).await?;
                                connection.write_frame(&ClockingFrameUnit::OneshotHeaders(oneshot.oneshot_header)).await?;
                                connection.write_frame(&ClockingFrameUnit::Content(oneshot.payload)).await?;
                            }
                        }
                    },
                    read = connection.read_frame() => {
                        match read {
                            Ok(Some(payload)) => {
                                if let Some(received) = frame_buffer.append(payload, MessageAuthor::Server) {
                                    if received.sutera_status.is_none() {
                                        panic!("Received message doesn't contain sutera_header! (Maybe frame_buffer has bugs.)")
                                    }
                                    match received.content_header {
                                        ContentHeader::Event(event_header) if event_header.message_type == EventTypes::TextChat_ReceiveChatMessage_Push => {
                                            let chat_message = deserialize::<SendableChatEntry, SendableChatEntry>(&received.payload)?; Gd::<ClockerConnection>::from_instance_id(instance_id).cast::<ClockerConnection>().call_deferred(
                                                "emit_signal".into(),
                                                &[
                                                    Variant::from(SIGNAL_NEW_TEXTCHAT_MESSAGE.into_godot()),
                                                    Variant::from(chat_message.sender.into_godot()),
                                                    Variant::from(chat_message.message.into_godot()),
                                                ] ,
                                            );

                                        }
                                        ContentHeader::Event(event_header) => {
                                            receive.send(
                                                Request::Event(EventMessage::new(
                                                    received.sutera_header,
                                                    event_header,
                                                    received.payload,
                                                ))
                                            ).await.map_err(TcpServerError::CannotSendRequest)?;
                                        }
                                        ContentHeader::Oneshot(oneshot_header) => {
                                            // FIXME: Push型かPull型かチェックしてあげないといけない
                                            if let Entry::Occupied(o) = reply_senders.entry(oneshot_header.message_id) {
                                                o.remove_entry().1.send(Request::Oneshot(OneshotRequest::new(
                                                    received.sutera_header,
                                                    received.sutera_status.unwrap(),
                                                    oneshot_header,
                                                    received.payload,
                                                ))).map_err(|_| TcpServerError::CannotSendOneshotReply)?;
                                            } else {
                                                receive.send(
                                                    Request::Oneshot(OneshotRequest::new(
                                                        received.sutera_header,
                                                        received.sutera_status.unwrap(),
                                                        oneshot_header,
                                                        received.payload,
                                                    ))
                                                ).await.map_err(TcpServerError::CannotSendRequest)?;
                                            }
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
                    _shutdown = &mut shutdown_rx => {
                        break;
                    }
                }
            }

            connection.shutdown_stream().await?;
            Ok::<(), TcpServerError>(())
        };

        let handle_logger = logger.clone();
        let handle = tokio().bind().spawn("clocker_connection", async move {
            match server.await {
                Ok(_) => info!(handle_logger, "Connection successfully finished."),
                Err(e) => warn!(handle_logger, "Connection failed: {}", e),
            }
        });

        Self {
            shutdown_tx,
            _receive_rx: receive_rx,
            send_tx,
            handle,
        }
    }
}
