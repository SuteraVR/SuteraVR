use rustls_pemfile::{certs, pkcs8_private_keys};
use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};
use tokio_rustls::rustls::{
    self,
    pki_types::{CertificateDer, PrivateKeyDer},
    ServerConfig,
};

pub struct SingleCerts {
    pub certs: Vec<CertificateDer<'static>>,
    pub keys: PrivateKeyDer<'static>,
}

impl SingleCerts {
    pub fn new(certs_path: &Path, keys_path: &Path) -> io::Result<Self> {
        Ok(Self {
            certs: certs(&mut BufReader::new(File::open(certs_path)?))
                .collect::<io::Result<Vec<CertificateDer<'static>>>>()?,
            keys: pkcs8_private_keys(&mut BufReader::new(File::open(keys_path)?))
                .next()
                .ok_or(io::ErrorKind::InvalidInput)?
                .map(Into::into)?,
        })
    }
    pub fn gen_server_config(self) -> io::Result<ServerConfig> {
        rustls::ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
            .with_no_client_auth()
            .with_single_cert(self.certs, self.keys)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
    }
}
