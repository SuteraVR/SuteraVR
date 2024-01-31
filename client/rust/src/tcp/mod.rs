pub mod allow_unknown_cert;
pub mod error;

use std::sync::Arc;

use futures::executor::block_on;
use godot::{engine::notify::NodeNotification, prelude::*};
use suteravr_lib::{
    clocking::{
        buffer::FrameBuffer,
        oneshot_headers::{OneshotHeader, OneshotStep, OneshotTypes},
        sutera_header::SuteraHeader,
        traits::MessageAuthor,
        ClockingConnection, ClockingFrameUnit,
    },
    info, warn, SCHEMA_VERSION,
};
use tokio::{
    net::TcpStream,
    sync::oneshot,
    task::{JoinError, JoinHandle},
};
use tokio_rustls::{
    rustls::{pki_types::ServerName, ClientConfig},
    TlsConnector,
};

use crate::{
    async_driver::tokio,
    logger::GodotLogger,
    tcp::{allow_unknown_cert::AllowUnknownCertVerifier, error::TcpServerError},
};

#[derive(Debug)]
enum ShutdownReason {
    GameExit,
}

#[derive(GodotClass)]
#[class(base=Node)]
struct ClockerConnection {
    base: Base<Node>,
    logger: GodotLogger,
    shutdown_tx: Option<oneshot::Sender<ShutdownReason>>,
    handle: Option<JoinHandle<()>>,
}

impl ClockerConnection {
    fn logger(&self) -> GodotLogger {
        self.logger.clone()
    }

    async fn shutdown(&mut self) -> Result<(), JoinError> {
        if let Some(tx) = self.shutdown_tx.take() {
            tx.send(ShutdownReason::GameExit).unwrap();
        }
        if let Some(handle) = self.handle.take() {
            handle.await?;
        }
        Ok(())
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
            handle: None,
            shutdown_tx: None,
        }
    }

    fn ready(&mut self) {
        info!(self.logger, "Making connection...");

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<ShutdownReason>();
        self.shutdown_tx = Some(shutdown_tx);

        let logger = self.logger();
        let server = async move {
            let name = "localhost";
            let addr = "127.0.0.1:3501";

            info!(logger, "Connecting to {}({}) ...", name, addr);

            let config = ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(AllowUnknownCertVerifier::new())
                .with_no_client_auth();
            warn!(logger, "Allowing unknown certificates.");
            warn!(logger, "Ensure you are connecting to the right server!");

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

            connection
                .write_frame(&ClockingFrameUnit::SuteraHeader(SuteraHeader {
                    version: SCHEMA_VERSION,
                }))
                .await?;
            connection
                .write_frame(&ClockingFrameUnit::OneshotHeaders(OneshotHeader {
                    step: OneshotStep::Request,
                    message_type: OneshotTypes::Connection_HealthCheck_Pull,
                    message_id: 0x123,
                }))
                .await?;
            connection
                .write_frame(&ClockingFrameUnit::Content(vec![]))
                .await?;

            loop {
                tokio::select! {
                    read = connection.read_frame() => {
                        match read {
                            Ok(Some(payload)) => {
                                if let Some(_received) = frame_buffer.append(payload, MessageAuthor::Server) {
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

        let logger = self.logger();
        self.handle = Some(tokio().bind().spawn("clocker_connection", async move {
            match server.await {
                Ok(_) => info!(logger, "Connection successfully finished."),
                Err(e) => warn!(logger, "Connection failed: {}", e),
            }
        }));
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
