pub mod allow_unknown_cert;
pub mod error;
pub mod requests;

use alkahest::deserialize;
use srv_rs::{resolver::libresolv::LibResolv, SrvClient};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{atomic::AtomicU64, Arc, Mutex},
};
use suteravr_lib::{
    clocking::{
        event_headers::EventTypes,
        schemas::oneshot::{
            chat_entry::{SendChatMessageRequest, SendableChatEntry},
            login::{LoginRequest, LoginResponse},
        },
    },
    error,
    util::serialize_to_new_vec,
};

use futures::executor::block_on;
use godot::{engine::notify::NodeNotification, obj::WithBaseField, prelude::*};
use suteravr_lib::{
    clocking::{
        buffer::{ContentHeader, FrameBuffer},
        oneshot_headers::{OneshotHeader, OneshotStep, OneshotTypes},
        sutera_header::SuteraHeader,
        traits::MessageAuthor,
        ClockingConnection, ClockingFrameUnit,
    },
    info,
    messaging::id::MessageId,
    warn, SCHEMA_VERSION,
};
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
    task::{JoinError, JoinHandle},
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
        allow_unknown_cert::AllowUnknownCertVerifier,
        error::TcpServerError,
        requests::{EventMessage, OneshotRequest, OneshotResponse},
    },
};

use self::requests::{Request, Response};

#[derive(Debug)]
enum ShutdownReason {
    GameExit,
}

#[derive(GodotClass)]
#[class(base=Node)]
struct ClockerConnection {
    base: Base<Node>,
    logger: GodotLogger,
    connection: Arc<Mutex<Option<Connection>>>,
    message_id_dispatch: AtomicU64,
}

struct Connection {
    shutdown_tx: oneshot::Sender<ShutdownReason>,
    _receive_rx: mpsc::Receiver<Request>,
    send_tx: mpsc::Sender<Response>,
    handle: JoinHandle<()>,
}

impl ClockerConnection {
    fn logger(&self) -> GodotLogger {
        self.logger.clone()
    }

    async fn shutdown(&mut self) -> Result<(), JoinError> {
        let taken_connection = { self.connection.lock().unwrap().take() };
        if let Some(connection) = taken_connection {
            connection
                .shutdown_tx
                .send(ShutdownReason::GameExit)
                .unwrap();
            connection.handle.await?;
        }
        Ok(())
    }
}

impl Connection {
    fn new<T: Into<String> + Send + 'static>(
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

#[godot_api]
impl ClockerConnection {
    #[func]
    fn signal_new_textchat_message(&mut self) -> String {
        SIGNAL_NEW_TEXTCHAT_MESSAGE.to_string()
    }

    #[func]
    fn connect_by_srv(&mut self, domain: String) {
        let instance_id = self.base().instance_id();
        let logger = self.logger();
        let connection = self.connection.clone();
        tokio().bind().spawn("connect_by_srv", async move {
            let client = SrvClient::<LibResolv>::new(format!("_suteravr-clocker._tls.{}", domain));
            let srv = client
                .execute(srv_rs::Execution::Concurrent, |address| async move {
                    Ok::<_, std::convert::Infallible>(address.authority().map(|v| v.to_string()))
                })
                .await;
            if let Ok(Ok(Some(e))) = srv {
                info!(logger, "SRV record resolved: {:?}", e);
                let mut root_cert_store = tokio_rustls::rustls::RootCertStore::empty();
                for cert in rustls_native_certs::load_native_certs().unwrap() {
                    root_cert_store.add(cert).unwrap();
                }
                let config = ClientConfig::builder()
                    .with_root_certificates(root_cert_store)
                    .with_no_client_auth();
                Self::connect(connection, logger, instance_id, config, domain, e)
            } else {
                error!(logger, "Failed to resolve SRV record: {}", srv.unwrap_err());
            }
        });
    }

    #[func]
    fn connect_to_localhost(&mut self) {
        self.connect_to_localhost_with_port(3501)
    }

    #[func]
    fn connect_to_localhost_with_port(&mut self, port: u16) {
        self.connect_without_certification_verifying(
            "localhost".into(),
            format!("127.0.0.1:{}", port),
        )
    }

    #[func]
    fn connect_without_certification_verifying(&mut self, name: String, addr: String) {
        let config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(AllowUnknownCertVerifier::new())
            .with_no_client_auth();
        warn!(self.logger, "Allowing unknown certificates.");
        warn!(
            self.logger,
            "Ensure you are connecting to the right server!"
        );
        Self::connect(
            self.connection.clone(),
            self.logger.clone(),
            self.base().instance_id(),
            config,
            name,
            addr,
        );
    }

    #[func]
    fn oneshot_send_chat_message(&mut self, content: String) {
        let id = self.get_message_id();
        let logger = self.logger();
        let send = self.send_tx();
        tokio().bind().spawn("clocking_request", async move {
            let login_result = Self::create_oneshot_p(
                logger.clone(),
                send,
                OneshotResponse {
                    sutera_header: SuteraHeader {
                        version: SCHEMA_VERSION,
                    },
                    oneshot_header: OneshotHeader {
                        step: OneshotStep::Request,
                        message_type: OneshotTypes::TextChat_SendMessage_Pull,
                        message_id: id,
                    },

                    payload: serialize_to_new_vec(SendChatMessageRequest { content }),
                },
            )
            .await?;
            let result = deserialize::<LoginResponse, LoginResponse>(&login_result.payload)?;
            info!(logger, "ChatMessage sent: {:?}", result);
            Ok::<(), TcpServerError>(())
        });
    }

    #[func]
    fn join_instance(&mut self, join_token: u64) {
        let id = self.get_message_id();
        let logger = self.logger();
        let send = self.send_tx();
        tokio().bind().spawn("clocking_request", async move {
            info!(logger, "Joining instance with token: {}", join_token);
            let login_result = Self::create_oneshot_p(
                logger.clone(),
                send,
                OneshotResponse {
                    sutera_header: SuteraHeader {
                        version: SCHEMA_VERSION,
                    },
                    oneshot_header: OneshotHeader {
                        step: OneshotStep::Request,
                        message_type: OneshotTypes::Authentication_Login_Pull,
                        message_id: id,
                    },

                    payload: serialize_to_new_vec(LoginRequest { join_token }),
                },
            )
            .await?;
            let result = deserialize::<LoginResponse, LoginResponse>(&login_result.payload)?;
            info!(logger, "Instance Joined: {:?}", result);
            Ok::<(), TcpServerError>(())
        });
    }
}
impl ClockerConnection {
    fn send_tx(&self) -> mpsc::Sender<Response> {
        return self
            .connection
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .send_tx
            .clone();
    }

    fn connect<T: Into<String> + Send + 'static>(
        connection: Arc<Mutex<Option<Connection>>>,
        logger: GodotLogger,
        instance_id: InstanceId,
        config: ClientConfig,
        name: T,
        addr: T,
    ) {
        *connection.lock().unwrap() =
            Some(Connection::new(logger, instance_id, config, name, addr));
    }

    async fn create_oneshot_p(
        logger: GodotLogger,
        send: mpsc::Sender<Response>,
        response: OneshotResponse,
    ) -> Result<OneshotRequest, TcpServerError> {
        let message_id = response.oneshot_header.message_id;
        let (tx, rx) = oneshot::channel::<Request>();
        send.send(Response::OneshotWithReply(response, tx))
            .await
            .map_err(TcpServerError::CannotSendResponse)?;

        // TODO: そのうちRequestにOneshot以外が実装されるので、irrefutable_let_patternsは解消されるはず
        #[allow(irrefutable_let_patterns)]
        let Request::Oneshot(oneshot) = rx.await?
        else {
            error!(
                logger,
                "rx of messageId {:?} not received Oneshot!", message_id
            );
            panic!();
        };
        Ok(oneshot)
    }
    fn get_message_id(&mut self) -> MessageId {
        self.message_id_dispatch
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed) as MessageId
    }
}

#[godot_api]
impl INode for ClockerConnection {
    fn init(base: Base<Node>) -> Self {
        let logger = GodotLogger {
            target: "ClockerConnection".to_string(),
        };
        Self {
            base,
            logger,
            connection: Arc::new(Mutex::new(None)),
            message_id_dispatch: AtomicU64::new(0),
        }
    }

    fn ready(&mut self) {
        self.base_mut()
            .add_user_signal(SIGNAL_NEW_TEXTCHAT_MESSAGE.into());
    }

    fn on_notification(&mut self, what: NodeNotification) {
        match what {
            NodeNotification::WmCloseRequest | NodeNotification::ExitTree => {
                info!(self.logger, "Shutting down...");
                block_on(self.shutdown()).unwrap();
                info!(self.logger, "Shutdown complete.");
            }
            _ => {}
        }
    }
}
