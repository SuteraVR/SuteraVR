//! Clocking-server
//!
//! [`suteravr-lib`][suteravr_lib]が使用できます。
//! ```no_run
//! use suteravr_lib::Foo;
//! ```
//!
use std::{net::SocketAddr, path::PathBuf};

use errors::ClockingServerError;
use log::{error, info};
use tokio::{
    sync::{mpsc, oneshot},
    task,
};
use tokio_rustls::rustls::ServerConfig;

use crate::{
    shutdown::ShutdownReason,
    signal::listen_signal,
    tcp::{certs::SingleCerts, tcp_server, TcpServerSignal},
};

mod consts;
pub mod errors;
mod shutdown;
mod signal;
mod tcp;
pub mod instance;

pub async fn clocking_server() -> Result<(), ClockingServerError> {
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
    let (shutdown_tx, shutdown) = oneshot::channel::<ShutdownReason>();

    let server = task::Builder::new()
        .name("TCP server")
        .spawn(tcp_server(cfg, addr, tcp_rx))
        .map_err(ClockingServerError::SpawnError)?;

    let signal = task::Builder::new()
        .name("Signal listener")
        .spawn(listen_signal(shutdown_tx))
        .map_err(ClockingServerError::SpawnError)?;

    let reason = match shutdown.await {
        Ok(reason) => {
            info!("Doing graceful shutdown: {:?}", reason);
            reason
        }
        Err(e) => {
            error!("Failed to receive shutdown signal: {}", e);
            ShutdownReason::SignalChannelClosed
        }
    };

    let _ = tcp_tx
        .send(TcpServerSignal::Shutdown(reason))
        .await
        .map_err(|_| error!("Failed to send shutdown signal to TCP server"));

    server.await??;
    signal.await??;

    Ok(())
}
