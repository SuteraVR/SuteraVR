use std::{net::SocketAddr, path::PathBuf};
use tokio::{
    sync::{mpsc, oneshot},
    task,
};

use log::{error, info};
use tokio_rustls::rustls::ServerConfig;

use crate::{
    consts,
    shutdown::ShutdownReason,
    signal::listen_signal,
    tcp::{certs::SingleCerts, error::ClockingServerError, tcp_server, TcpServerSignal},
};
