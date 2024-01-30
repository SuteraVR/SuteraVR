pub mod certs;
pub mod requests;
pub mod stream;

use log::error;
use log::{info, warn};
use std::{io, net::SocketAddr, sync::Arc};
use suteravr_lib::clocking::oneshot_headers::OneshotTypes;
use suteravr_lib::clocking::sutera_status::{SuteraStatus, SuteraStatusError};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{broadcast, mpsc::Receiver},
    task::JoinSet,
};
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};

use crate::errors::TcpServerError;
use crate::shutdown::ShutdownReason;
use crate::tcp::requests::Request;
use crate::tcp::stream::ClientMessageStream;

#[derive(Debug)]
pub enum TcpServerSignal {
    Shutdown(ShutdownReason),
}

pub async fn tcp_server(
    cfg: ServerConfig,
    addr: SocketAddr,
    mut rx: Receiver<TcpServerSignal>,
) -> Result<(), TcpServerError> {
    let acceptor = &TlsAcceptor::from(Arc::new(cfg));
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(TcpServerError::ListenerBindError)?;

    info!("Ready! Server running on {}", &addr);

    let mut connections = JoinSet::new();
    let (shutdown_tx, _) = broadcast::channel::<ShutdownReason>(1);

    let shutdown_reason = 'accepting: loop {
        tokio::select! {
            signal = rx.recv() => {
                match signal {
                    Some(TcpServerSignal::Shutdown(reason)) => {
                        break 'accepting reason;
                    }
                    None => {
                        warn!("Signal channel is closed.");
                        break 'accepting ShutdownReason::SignalChannelClosed;
                    }
                }
            }
            accepted = listener.accept() => {
                connection_init(accepted, acceptor, &mut connections, shutdown_tx.subscribe()).await?;
            }
        }
    };

    if !connections.is_empty() {
        info!("Waiting for all connections to be closed...");
        match shutdown_tx.send(shutdown_reason) {
            Ok(_) => while (connections.join_next().await).is_some() {},
            Err(e) => {
                error!("Failed to send shutdown signal: {}", e);
                warn!("Shutting down immediately...");
                connections.shutdown().await;
            }
        }
    }
    info!("Shutting down... (tcp)");
    Ok(())
}

async fn connection_init(
    accepted: io::Result<(TcpStream, SocketAddr)>,
    acceptor: &TlsAcceptor,
    join_set: &mut JoinSet<()>,
    mut shutdown_rx: broadcast::Receiver<ShutdownReason>,
) -> Result<(), TcpServerError> {
    let Ok((stream, peer_addr)) = accepted else {
        if let Err(e) = accepted {
            warn!("Failed to accept connection: {}", e);
        }
        return Ok(());
    };

    let acceptor = acceptor.clone();
    info!("Connection from {}...", peer_addr);

    let fut = async move {
        let stream = acceptor
            .accept(stream)
            .await
            .map_err(TcpServerError::AcceptError)?;
        info!("Connection from {} is established.", peer_addr);

        let (mut message, mut stream_handle) = ClientMessageStream::new(stream, peer_addr)?;
        loop {
            tokio::select! {
                _ = &mut stream_handle => {
                    break;
                },
                Some(request) = message.recv() => {
                    match request {
                        Request::Oneshot(request) if request.oneshot_header.message_type == OneshotTypes::Connection_HealthCheck_Pull => {
                            request.reply(Vec::new());
                        }
                        Request::Oneshot(request) => {
                            request.reply_failed(SuteraStatus::Error(SuteraStatusError::Unimplemented));
                        }
                    }
                },
                Ok(reason) = shutdown_rx.recv() => {
                    message.shutdown(reason).await?;
                    break;
                }
            }
        }
        stream_handle.await??;
        Ok::<(), TcpServerError>(())
    };

    join_set
        .build_task()
        .name(format!("Acceptor {}", peer_addr).as_str())
        .spawn(async move {
            match fut.await {
                Ok(_) => {
                    info!("Connection from {} is closed.", peer_addr);
                }
                Err(e) => {
                    warn!("Failed in Acceptor {} ({})", peer_addr, e);
                }
            }
        })
        .map_err(TcpServerError::SpawnError)?;

    Ok(())
}
