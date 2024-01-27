use std::{io, net::SocketAddr, path::PathBuf, sync::Arc};

use clocking_server::{certs::SingleCerts, consts};
use log::{error, info};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task,
};
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("====================");
    info!("SuteraVR / Clocking-server");
    info!("Version: {}", consts::VERSION);
    info!("====================");
    info!("");
    match *consts::ENV {
        consts::SuteraEnv::Development => {
            info!("Running in Development mode...");
            console_subscriber::init();
            info!("console_subscriber initialized. To debug, tokio-console may help you.");
        }
        consts::SuteraEnv::Production => info!("Running in Production mode..."),
    }

    info!("");
    info!("Loading Certifications...");
    let single_certs = SingleCerts::new(
        &PathBuf::from("./certs/server.crt"),
        &PathBuf::from("./certs/server.key"),
    )
    .map_err(|e| {
        error!("Failed to load certifications!: {}", e);
        error!("Ensure that ./server.crt and ./private.pem exists.");
        info!("Hint: you can generate your own by certgen.sh");
        e
    })?;

    let cfg: ServerConfig = single_certs.gen_server_config()?;

    info!("");

    let addr = SocketAddr::from(([127, 0, 0, 1], *consts::PORT));

    let acceptor = &TlsAcceptor::from(Arc::new(cfg));
    let listener = TcpListener::bind(&addr).await?;

    info!("Ready! Server running on {}", &addr);
    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        info!("Connection from {}...", peer_addr);

        let fut = async move {
            let mut stream = acceptor.accept(stream).await?;
            info!("Connection from {} is established.", peer_addr);

            let mut buf = vec![0; 1024];
            while let Ok(n) = stream.read(&mut buf).await {
                if n == 0 {
                    break;
                }
                let string = String::from_utf8_lossy(&buf[..n]);
                info!("Received from {}: {}", peer_addr, string.trim_end());
                stream
                    .write_all(format!("Received: {}", string).as_bytes())
                    .await?;
            }

            info!("Connection from {} is closed.", peer_addr);
            io::Result::Ok(())
        };
        task::Builder::new()
            .name(format!("Acceptor {}", peer_addr).as_str())
            .spawn(fut)
            .unwrap();
    }
}
