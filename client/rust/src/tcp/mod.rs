pub mod allow_unknown_cert;

use std::sync::Arc;

use godot::prelude::*;
use suteravr_lib::{info, warn};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_rustls::{
    rustls::{pki_types::ServerName, ClientConfig},
    TlsConnector,
};

use crate::{
    async_driver::tokio, logger::GodotLogger, tcp::allow_unknown_cert::AllowUnknownCertVerifier,
};

#[derive(GodotClass)]
#[class(base=Node)]
struct ClockerConnection {
    base: Base<Node>,
    logger: GodotLogger,
}

impl ClockerConnection {
    fn logger(&self) -> GodotLogger {
        self.logger.clone()
    }
}

#[godot_api]
impl INode for ClockerConnection {
    fn init(base: Base<Node>) -> Self {
        let logger = GodotLogger {
            target: "ClockerConnection".to_string(),
        };
        Self { base, logger }
    }

    fn ready(&mut self) {
        info!(self.logger, "Making connection...");

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
            stream.shutdown().await?;

            Ok::<(), std::io::Error>(())
        };

        let logger = self.logger();
        tokio().bind().spawn("clocker_connection", async move {
            match server.await {
                Ok(_) => info!(logger, "Connection successfully finished."),
                Err(e) => warn!(logger, "Connection failed: {}", e),
            }
        });
    }
}
