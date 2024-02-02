pub mod certs;
pub mod requests;
pub mod stream;

use alkahest::deserialize;
use chrono::Local;
use log::error;
use log::{info, warn};
use std::{io, net::SocketAddr, sync::Arc};
use suteravr_lib::clocking::event_headers::EventTypes;
use suteravr_lib::clocking::oneshot_headers::OneshotTypes;
use suteravr_lib::clocking::schemas::oneshot::chat_entry::{
    ChatEntry, SendChatMessageRequest, SendChatMessageResponse, SendableChatEntry,
};
use suteravr_lib::clocking::schemas::oneshot::login::{LoginRequest, LoginResponse};
use suteravr_lib::clocking::sutera_status::{SuteraStatus, SuteraStatusError};
use suteravr_lib::messaging::id::PlayerId;
use tokio::sync::{mpsc, oneshot};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::{broadcast, mpsc::Receiver},
    task::JoinSet,
};
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};

use crate::errors::TcpServerError;
use crate::instance::manager::InstancesControl;
use crate::instance::{InstanceControl, PlayerControl};
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
    instances_tx: mpsc::Sender<InstancesControl>,
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
                connection_init(accepted, acceptor, &mut connections, shutdown_tx.subscribe(), instances_tx.clone()).await?;
            }
        }
    };

    if shutdown_tx.receiver_count() > 0 {
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
    instances_tx: mpsc::Sender<InstancesControl>,
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

        let mut login_status: Option<(PlayerId, mpsc::Sender<InstanceControl>)> = None;
        let (control_tx, mut control) = mpsc::channel::<PlayerControl>(32);

        let (mut message, mut stream_handle) = ClientMessageStream::new(stream, peer_addr)?;
        loop {
            tokio::select! {
                Some(control) = control.recv() => {
                    match control {
                        PlayerControl::NewChatMessage(entry) => {
                            message.send_event_ok(EventTypes::TextChat_ReceiveChatMessage_Push,
                            SendableChatEntry::from(entry)).await?;
                        }
                    }
                },
                Some(request) = message.recv() => {
                    match request {
                        Request::Oneshot(request) if request.oneshot_header.message_type == OneshotTypes::Connection_HealthCheck_Pull => {
                            request.send_reply(Vec::new()).await?;
                        }
                        Request::Oneshot(request) if request.oneshot_header.message_type == OneshotTypes::Authentication_Login_Pull => {
                            let Ok(payload) = deserialize::<LoginRequest, LoginRequest>(&request.payload) else {
                                request.send_reply_bad_request().await?;
                                continue;
                            };
                            let (reply, reply_recv) = oneshot::channel();
                            instances_tx.send(InstancesControl::JoinInstance { id: payload.join_token, reply, control: control_tx.clone() }).await?;
                            if let Some(auth) = reply_recv.await.map_err(TcpServerError::CannotReceiveFromInstanceManager)? {
                                login_status = Some(auth);
                                request.serialize_and_send_reply(LoginResponse::Ok).await?;
                            } else {
                                request.serialize_and_send_reply(LoginResponse::BadToken).await?;
                            }
                        }
                        Request::Oneshot(request) if request.oneshot_header.message_type == OneshotTypes::TextChat_SendMessage_Pull => {
                            let Ok(payload) = deserialize::<SendChatMessageRequest, SendChatMessageRequest>(&request.payload) else {
                                request.send_reply_bad_request().await?;
                                continue;
                            };
                            let Some((player_id, instance_tx)) = &login_status else {
                                request.send_reply_unauthorized().await?;
                                continue;
                            };

                            let entry = ChatEntry {
                                send_at: Local::now(),
                                sender: *player_id,
                                message: payload.content,
                            };

                            instance_tx.send(InstanceControl::ChatMesasge(entry)).await?;
                            request.serialize_and_send_reply(SendChatMessageResponse::Ok).await?;

                        }
                        Request::Oneshot(request) => {
                            request.send_reply_failed(SuteraStatus::Error(SuteraStatusError::Unimplemented)).await?;
                        },
                        Request::Event(event) => {
                            error!("Received unexpected event: {:?}, skipping...", event);
                        }

                    }
                },
                _ = &mut stream_handle => {
                    break;
                },
                Ok(reason) = shutdown_rx.recv() => {
                    message.shutdown(reason).await?;
                    stream_handle.await??; break;
                }
            }
        }
        if let Some((player_id, instance_tx)) = login_status {
            instance_tx.send(InstanceControl::Leave(player_id)).await?;
        }

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
