use crate::tcp::allow_unknown_cert::AllowUnknownCertVerifier;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::ClientConfig;
use tokio_rustls::TlsConnector;
mod tcp;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:3501";

    let config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(AllowUnknownCertVerifier::new())
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let dnsname = ServerName::try_from("localhost").unwrap();

    let stream = TcpStream::connect(&addr).await?;
    let mut stream = connector.connect(dnsname, stream).await?;
    Ok(())
}
