use std::{env, io, path::PathBuf};

use clocking_server::{certs::SingleCerts, consts};
use log::{error, info};
use tokio_rustls::rustls::ServerConfig;

#[tokio::main]
async fn main() -> io::Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    info!("====================");
    info!("SuteraVR / Clocking-server");
    info!("Version: {}", consts::VERSION);
    info!("====================");
    info!("");

    info!("Loading Certifications...");
    let single_certs = SingleCerts::new(
        &PathBuf::from("./ssl_certs/server.crt"),
        &PathBuf::from("./ssl_certs/server.key"),
    ).map_err(|e| {
        error!("Failed to load certifications!: {}", e);
        error!("Ensure that ./server.crt and ./private.pem exists.");
        info!("Hint: you can generate your own by certgen.sh");
        e
    })?;

    let cfg: ServerConfig = single_certs.gen_server_config()?;

    info!("");
    info!("Server running on port :{}", *consts::PORT);

    Ok(())
}
