pub mod allow_unknown_cert;
pub mod conenction;
pub mod error;
pub mod requests;

use alkahest::deserialize;
use hickory_resolver::TokioAsyncResolver;
use std::sync::{atomic::AtomicU64, Arc, Mutex};
use suteravr_lib::{
    clocking::{
        event_headers::{EventDirection, EventHeader, EventTypes},
        schemas::{
            event::player_move::PubPlayerMove,
            oneshot::{
                chat_entry::SendChatMessageRequest,
                login::{LoginRequest, LoginResponse},
            },
        },
    },
    debug, error,
    messaging::player::{StandingTransform, StandingTransformEncoder},
    util::serialize_to_new_vec,
};

use futures::executor::block_on;
use godot::{engine::notify::NodeNotification, obj::WithBaseField, prelude::*};
use suteravr_lib::{
    clocking::{
        oneshot_headers::{OneshotHeader, OneshotStep, OneshotTypes},
        sutera_header::SuteraHeader,
    },
    info,
    messaging::id::MessageId,
    warn, SCHEMA_VERSION,
};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinError,
};
use tokio_rustls::rustls::ClientConfig;

use crate::{
    async_driver::tokio,
    logger::GodotLogger,
    signal_names::{
        SIGNAL_CONNECTION_ESTABLISHED, SIGNAL_NEW_TEXTCHAT_MESSAGE, SIGNAL_PLAYER_MOVED,
        SIGNAL_UPDATE_PLAYER_BEING,
    },
    tcp::{
        allow_unknown_cert::AllowUnknownCertVerifier,
        error::TcpServerError,
        requests::{OneshotRequest, OneshotResponse},
    },
};

use self::{
    conenction::Connection,
    requests::{EventMessage, Request, Response},
};

#[derive(Debug)]
pub enum ShutdownReason {
    GameExit,
}

#[derive(GodotClass)]
#[class(base=Node)]
struct ClockerConnection {
    base: Base<Node>,
    pos: StandingTransformEncoder,
    logger: GodotLogger,
    connection: Arc<Mutex<Option<Connection>>>,
    message_id_dispatch: AtomicU64,
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

#[godot_api]
impl ClockerConnection {
    #[func]
    fn signal_new_textchat_message(&mut self) -> String {
        SIGNAL_NEW_TEXTCHAT_MESSAGE.to_string()
    }
    #[func]
    fn signal_connection_established(&mut self) -> String {
        SIGNAL_CONNECTION_ESTABLISHED.to_string()
    }
    #[func]
    fn signal_update_player_being(&mut self) -> String {
        SIGNAL_UPDATE_PLAYER_BEING.to_string()
    }
    #[func]
    fn signal_player_moved(&mut self) -> String {
        SIGNAL_PLAYER_MOVED.to_string()
    }

    #[func]
    fn connect_by_srv(&mut self, domain: String) {
        let instance_id = self.base().instance_id();
        let logger = self.logger();
        let connection = self.connection.clone();
        tokio().bind().spawn("connect_by_srv", async move {
            let resolver = TokioAsyncResolver::tokio_from_system_conf().unwrap();
            let srv = resolver
                .srv_lookup(format!("_suteravr-clocker._tls.{}", domain))
                .await;
            match srv.map(|v| v.into_iter().next()) {
                Ok(Some(e)) => {
                    info!(logger, "SRV record resolved: {:?}", e);
                    let mut root_cert_store = tokio_rustls::rustls::RootCertStore::empty();
                    for cert in rustls_native_certs::load_native_certs().unwrap() {
                        root_cert_store.add(cert).unwrap();
                    }
                    let config = ClientConfig::builder()
                        .with_root_certificates(root_cert_store)
                        .with_no_client_auth();
                    Self::connect(
                        connection,
                        logger,
                        instance_id,
                        config,
                        domain,
                        format!("{}:{}", e.target(), e.port()),
                    )
                }
                Ok(None) => {
                    error!(
                        logger,
                        "Failed to resolve SRV record: SRV record not found."
                    );
                }
                Err(e) => {
                    error!(logger, "Failed to resolve SRV record: {:?}", e);
                }
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
        let Some(send) = self.send_tx() else {
            return;
        };
        tokio().bind().spawn("clocking_request", async move {
            let login_result = Self::create_oneshot_p(
                logger.clone(),
                send,
                OneshotRequest {
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
            debug!(logger, "ChatMessage sent: {:?}", result);
            Ok::<(), TcpServerError>(())
        });
    }

    #[func]
    fn report_player_transform(&mut self, x: f64, y: f64, z: f64, xx: f64, xz: f64) {
        self.pos.push(StandingTransform {
            x,
            y,
            z,
            yaw: (xx + 1f64) * xz.signum(),
        });
        if let Some(now) = self.pos.payload() {
            let Some(send) = self.send_tx() else {
                return;
            };
            tokio().bind().spawn("report_player_pos", async move {
                send.send(Request::Event(EventMessage {
                    sutera_header: SuteraHeader {
                        version: SCHEMA_VERSION,
                    },
                    event_header: EventHeader {
                        direction: EventDirection::Pull,
                        message_type: EventTypes::Instance_PubPlayerMove_Pull,
                    },
                    payload: serialize_to_new_vec(PubPlayerMove { now }),
                }))
                .await
                .map_err(TcpServerError::CannotSendRequest)?;
                Ok::<(), TcpServerError>(())
            });
        }
    }

    #[func]
    fn join_instance(&mut self, join_token: u64) {
        let id = self.get_message_id();
        let logger = self.logger();
        let Some(send) = self.send_tx() else {
            return;
        };
        let instance_id = self.base().instance_id();
        tokio().bind().spawn("clocking_request", async move {
            info!(logger, "Joining instance with token: {}", join_token);
            let login_result = Self::create_oneshot_p(
                logger.clone(),
                send,
                OneshotRequest {
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
            if let LoginResponse::Ok(players) = result {
                for player in players {
                    Gd::<ClockerConnection>::from_instance_id(instance_id)
                        .cast::<ClockerConnection>()
                        .call_deferred(
                            "emit_signal".into(),
                            &[
                                Variant::from(SIGNAL_UPDATE_PLAYER_BEING.into_godot()),
                                Variant::from(player.into_godot()),
                                Variant::from(true.into_godot()),
                                Variant::from(true.into_godot()),
                            ],
                        );
                }
            }
            Ok::<(), TcpServerError>(())
        });
    }
}
impl ClockerConnection {
    fn send_tx(&self) -> Option<mpsc::Sender<Request>> {
        Some(self.connection.lock().ok()?.as_ref()?.send_tx.clone())
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
        send: mpsc::Sender<Request>,
        response: OneshotRequest,
    ) -> Result<OneshotResponse, TcpServerError> {
        let message_id = response.oneshot_header.message_id;
        let (tx, rx) = oneshot::channel::<Response>();
        send.send(Request::OneshotWithReply(response, tx))
            .await
            .map_err(TcpServerError::CannotSendRequest)?;

        let Response::Oneshot(oneshot) = rx.await? else {
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
            pos: StandingTransformEncoder::new(),
            logger,
            connection: Arc::new(Mutex::new(None)),
            message_id_dispatch: AtomicU64::new(0),
        }
    }

    fn ready(&mut self) {
        self.base_mut()
            .add_user_signal(SIGNAL_NEW_TEXTCHAT_MESSAGE.into());
        self.base_mut()
            .add_user_signal(SIGNAL_CONNECTION_ESTABLISHED.into());
        self.base_mut()
            .add_user_signal(SIGNAL_UPDATE_PLAYER_BEING.into());
        self.base_mut().add_user_signal(SIGNAL_PLAYER_MOVED.into());
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
