use alkahest::deserialize;

use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};
use suteravr_lib::{
    clocking::{
        event_headers::EventTypes,
        oneshot_headers::{
            OneshotDirection, OneshotHeader, OneshotStep, OneshotTypes, ONESHOT_DIRECTION_MAP,
        },
        schemas::{
            event::update_player_being::{PlayerJoined, PlayerLeft},
            oneshot::chat_entry::SendableChatEntry,
        },
        sutera_header::SuteraHeader,
    },
    error, SCHEMA_VERSION,
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
    signal_names::{
        SIGNAL_CONNECTION_ESTABLISHED, SIGNAL_NEW_TEXTCHAT_MESSAGE, SIGNAL_UPDATE_PLAYER_BEING,
    },
    tcp::{
        error::TcpServerError,
        requests::{EventMessage, OneshotRequest, OneshotResponse},
        ClockerConnection,
    },
};

use super::{
    requests::{Request, Response},
    ShutdownReason,
};

pub struct Connection {
    pub shutdown_tx: oneshot::Sender<ShutdownReason>,
    pub _receive_rx: mpsc::Receiver<Response>,
    pub send_tx: mpsc::Sender<Request>,
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
        let (receive_tx, receive_rx) = mpsc::channel::<Response>(32);
        let (send_tx, mut send_rx) = mpsc::channel::<Request>(32);

        let server_logger = logger.clone();
        let reply = send_tx.clone();

        let server = async move {
            let mut reply_senders = HashMap::<MessageId, oneshot::Sender<Response>>::new();
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

            Gd::<ClockerConnection>::from_instance_id(instance_id)
                .cast::<ClockerConnection>()
                .call_deferred(
                    "emit_signal".into(),
                    &[Variant::from(SIGNAL_CONNECTION_ESTABLISHED.into_godot())],
                );

            let receive = receive_tx;

            async fn process_oneshot_response(
                response: OneshotResponse,
                reply: mpsc::Sender<Request>,
                logger: GodotLogger,
            ) -> Result<(), TcpServerError> {
                match response.oneshot_header.message_type {
                    OneshotTypes::Connection_HealthCheck_Push => {
                        let response = OneshotRequest {
                            sutera_header: SuteraHeader {
                                version: SCHEMA_VERSION,
                            },
                            oneshot_header: OneshotHeader {
                                step: OneshotStep::Response,
                                message_type: response.oneshot_header.message_type,
                                message_id: response.oneshot_header.message_id,
                            },
                            payload: Vec::new(),
                        };
                        reply
                            .send(Request::Oneshot(response))
                            .await
                            .map_err(TcpServerError::CannotSendRequest)?;
                    }
                    _ => {
                        error!(
                            logger,
                            "Unknown or unimplemented oneshot message type: {:?}",
                            response.oneshot_header.message_type
                        );
                        // WARNING:
                        // 現状クライアント側がUnimplementedを伝える手段が(スキーマ上)ないため、空のペイロードを返す
                        let response = OneshotRequest {
                            sutera_header: SuteraHeader {
                                version: SCHEMA_VERSION,
                            },
                            oneshot_header: OneshotHeader {
                                step: OneshotStep::Response,
                                message_type: response.oneshot_header.message_type,
                                message_id: response.oneshot_header.message_id,
                            },
                            payload: Vec::new(),
                        };
                        reply
                            .send(Request::Oneshot(response))
                            .await
                            .map_err(TcpServerError::CannotSendRequest)?;
                    }
                }
                Ok(())
            }

            loop {
                tokio::select! {
                    Some(request) = send_rx.recv() => {
                        match request {
                            Request::Oneshot(oneshot) => {
                                connection.write_frame(&ClockingFrameUnit::SuteraHeader(oneshot.sutera_header)).await?;
                                connection.write_frame(&ClockingFrameUnit::OneshotHeaders(oneshot.oneshot_header)).await?;
                                connection.write_frame(&ClockingFrameUnit::Content(oneshot.payload)).await?;
                            },
                            Request::OneshotWithReply(oneshot, sender) => {
                                let Entry::Vacant(o) = reply_senders.entry(oneshot.oneshot_header.message_id) else {
                                    error!(logger, "MessageId {:?} is already occupied!", oneshot.oneshot_header.message_id);
                                    panic!();
                                };
                                o.insert(sender);
                                connection.write_frame(&ClockingFrameUnit::SuteraHeader(oneshot.sutera_header)).await?;
                                connection.write_frame(&ClockingFrameUnit::OneshotHeaders(oneshot.oneshot_header)).await?;
                                connection.write_frame(&ClockingFrameUnit::Content(oneshot.payload)).await?;
                            },
                            Request::Event(event) => {
                                connection.write_frame(&ClockingFrameUnit::SuteraHeader(event.sutera_header)).await?;
                                connection.write_frame(&ClockingFrameUnit::EventHeader(event.event_header)).await?;
                                connection.write_frame(&ClockingFrameUnit::Content(event.payload)).await?;
                            },
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
                                            let chat_message = deserialize::<SendableChatEntry, SendableChatEntry>(&received.payload)?;
                                            Gd::<ClockerConnection>::from_instance_id(instance_id).cast::<ClockerConnection>().call_deferred(
                                                "emit_signal".into(),
                                                &[
                                                    Variant::from(SIGNAL_NEW_TEXTCHAT_MESSAGE.into_godot()),
                                                    Variant::from(chat_message.sender.into_godot()),
                                                    Variant::from(chat_message.message.into_godot()),
                                                ] ,
                                            );
                                        },
                                        ContentHeader::Event(event_header)  if event_header.message_type == EventTypes::Instance_PlayerJoined_Push => {
                                            let joined = deserialize::<PlayerJoined, PlayerJoined>(&received.payload)?;
                                            Gd::<ClockerConnection>::from_instance_id(instance_id).cast::<ClockerConnection>().call_deferred(
                                                "emit_signal".into(),
                                                &[
                                                    Variant::from(SIGNAL_UPDATE_PLAYER_BEING.into_godot()),
                                                    Variant::from(joined.joined_player.into_godot()),
                                                    Variant::from(true.into_godot()),
                                                ] ,
                                            );
                                        },
                                        ContentHeader::Event(event_header)  if event_header.message_type == EventTypes::Instance_PlayerLeft_Push => {
                                            let left = deserialize::<PlayerLeft, PlayerLeft>(&received.payload)?;
                                            Gd::<ClockerConnection>::from_instance_id(instance_id).cast::<ClockerConnection>().call_deferred(
                                                "emit_signal".into(),
                                                &[
                                                    Variant::from(SIGNAL_UPDATE_PLAYER_BEING.into_godot()),
                                                    Variant::from(left.left_player.into_godot()),
                                                    Variant::from(false.into_godot()),
                                                ] ,
                                            );
                                        },
                                        ContentHeader::Event(event_header) => {
                                            receive.send(
                                                Response::Event(EventMessage::new(
                                                    received.sutera_header,
                                                    event_header,
                                                    received.payload,
                                                ))
                                            ).await.map_err(TcpServerError::CannotSendResponse)?;
                                        }
                                        ContentHeader::Oneshot(oneshot_header) => {
                                            if ONESHOT_DIRECTION_MAP[oneshot_header.message_type] == OneshotDirection::Pull {
                                                if let Entry::Occupied(o) = reply_senders.entry(oneshot_header.message_id) {
                                                    o.remove_entry().1.send(Response::Oneshot(OneshotResponse::new(
                                                        received.sutera_header,
                                                        received.sutera_status.unwrap(),
                                                        oneshot_header,
                                                        received.payload,
                                                    ))).map_err(|_| TcpServerError::CannotSendOneshotReply)?;
                                                } else {
                                                    receive.send(
                                                        Response::Oneshot(OneshotResponse::new(
                                                            received.sutera_header,
                                                            received.sutera_status.unwrap(),
                                                            oneshot_header,
                                                            received.payload,
                                                        ))
                                                    ).await.map_err(TcpServerError::CannotSendResponse)?;
                                                }
                                                continue;
                                            }
                                            let response = OneshotResponse::new(
                                                    received.sutera_header,
                                                    received.sutera_status.unwrap(),
                                                    oneshot_header,
                                                    received.payload,
                                            );
                                            process_oneshot_response(response, reply.clone(), logger.clone()).await?;
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
