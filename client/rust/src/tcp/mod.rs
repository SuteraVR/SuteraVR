pub mod allow_unknown_cert;

use std::sync::Arc;

use futures::executor::block_on;
use godot::{engine::notify::NodeNotification, prelude::*};
use suteravr_lib::{info, warn};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::oneshot,
    task::{JoinError, JoinHandle},
};
use tokio_rustls::{
    rustls::{pki_types::ServerName, ClientConfig},
    TlsConnector,
};

use crate::{
    async_driver::tokio, logger::GodotLogger, tcp::allow_unknown_cert::AllowUnknownCertVerifier,
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

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<ShutdownReason>();
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

            let stream = TcpStream::connect(&addr).await?;
            let mut stream = connector.connect(dnsname, stream).await?;
            shutdown_rx.await.unwrap();
            stream.shutdown().await?;
            Ok::<(), std::io::Error>(())
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
