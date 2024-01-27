use std::{env, io, path::PathBuf};

use clocking_server::{certs::SingleCerts, consts};
use log::{error, info};

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
        &PathBuf::from("./cert.pem"),
        &PathBuf::from("./root-ca.key.pem"),
    ).map_err(|e| {
        error!("Failed to load certifications!: {}", e);
        error!("Ensure that ./server.crt and ./private.pem exists.");
        e
    })?;



    info!("Run on port :{}", *consts::PORT);

    Ok(())
}
