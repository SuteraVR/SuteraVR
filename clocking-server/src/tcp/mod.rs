pub mod certs;
pub mod error;

use log::error;
use log::{info, warn};
use std::{io, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc::Receiver,
    task,
};
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};

use crate::shutdown::ShutdownReason;

use self::error::TcpServerError;

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

    loop {
        tokio::select! {
            signal = rx.recv() => {
                match signal {
                    Some(TcpServerSignal::Shutdown(_)) => {
                        info!("Shutting down...");
                        break;
                    }
                    None => {
                        error!("Signal channel is closed. Shutting downe..");
                        break;
                    }
                }
            }
            accepted = listener.accept() => {
                connection_init(accepted, acceptor).await?;
            }
        }
    }
    Ok(())
}

async fn connection_init(
    accepted: io::Result<(TcpStream, SocketAddr)>,
    acceptor: &TlsAcceptor,
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

        io::Result::Ok(())
    };
    match task::Builder::new()
        .name(format!("Acceptor {}", peer_addr).as_str())
        .spawn(fut)
        .map_err(TcpServerError::SpawnError)?
        .await?
    {
        Ok(_) => {
            info!("Connection from {} is closed.", peer_addr);
        }
        Err(e) => {
            warn!("Failed to spawn acceptor for {} ({})", peer_addr, e);
        }
    }
    Ok(())
}
