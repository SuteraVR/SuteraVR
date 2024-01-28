use clocking_server::tcp::{error::ClockingServerError, tcp_server, TcpServerSignal};
use std::{net::SocketAddr, path::PathBuf};
use tokio::{sync::mpsc, task};

use clocking_server::{consts, tcp::certs::SingleCerts};
use log::{error, info};
use tokio_rustls::rustls::ServerConfig;

#[tokio::main]
async fn main() -> Result<(), ClockingServerError> {
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

    let (tcp_tx, tcp_rx) = mpsc::channel::<TcpServerSignal>(32);

    task::Builder::new()
        .name("TCP server")
        .spawn(tcp_server(cfg, addr, tcp_rx))
        .map_err(ClockingServerError::SpawnError)?
        .await?
        .map_err(ClockingServerError::TcpServerError)
}
