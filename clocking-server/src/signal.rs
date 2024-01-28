use log::info;
use tokio::signal::unix::signal;
use tokio::signal::unix::SignalKind;
use tokio::sync::oneshot;

use crate::shutdown::ShutdownReason;

pub async fn listen_signal(
    shutdown: oneshot::Sender<ShutdownReason>,
) -> Result<(), std::io::Error> {
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    tokio::select! {
        _ = sigterm.recv() => {
            info!("SIGTERM received");
            shutdown.send(ShutdownReason::SIGTERM).unwrap();
        }
        _ = sigint.recv() => {
            info!("SIGINT received");
            shutdown.send(ShutdownReason::SIGINT).unwrap();
        }
    }

    info!("Shutting down... (signal)");
    Ok(())
}
