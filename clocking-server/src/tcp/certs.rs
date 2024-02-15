use log::{error, info};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::{
    env,
    fs::File,
    io::{self, BufReader, Error, ErrorKind},
    path::PathBuf,
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

fn load_from_env() -> io::Result<Option<SingleCerts>> {
    let Ok(certs_env) = env::var("SINGLECERTS_CERT_PEM") else {
        return Ok(None);
    };
    let Ok(keys_env) = env::var("SINGLECERTS_KEY_PEM") else {
        return Ok(None);
    };

    let keys = pkcs8_private_keys(&mut keys_env.as_bytes())
        .next()
        .ok_or(io::ErrorKind::InvalidInput)?
        .map(Into::into)?;
    Ok(Some(SingleCerts {
        certs: certs(&mut certs_env.as_bytes())
            .collect::<io::Result<Vec<CertificateDer<'static>>>>()?,
        keys,
    }))
}

fn load_from_env_path() -> io::Result<Option<(SingleCerts, String, String)>> {
    let Ok(certs_path) = env::var("SINGLECERTS_CERT_PATH") else {
        return Ok(None);
    };
    let Ok(keys_path) = env::var("SINGLECERTS_KEY_PATH") else {
        return Ok(None);
    };
    let certs_file = &PathBuf::from(&certs_path);
    let keys_file = &PathBuf::from(&keys_path);
    Ok(Some((
        SingleCerts {
            certs: certs(&mut BufReader::new(File::open(certs_file)?))
                .collect::<io::Result<Vec<CertificateDer<'static>>>>()?,
            keys: pkcs8_private_keys(&mut BufReader::new(File::open(keys_file)?))
                .next()
                .ok_or(io::ErrorKind::InvalidInput)?
                .map(Into::into)?,
        },
        certs_path,
        keys_path,
    )))
}

fn load_from_file() -> io::Result<Option<SingleCerts>> {
    let certs_path = &PathBuf::from("./certs/server.crt");
    let keys_path = &PathBuf::from("./certs/server.key");
    Ok(Some(SingleCerts {
        certs: certs(&mut BufReader::new(File::open(certs_path)?))
            .collect::<io::Result<Vec<CertificateDer<'static>>>>()?,
        keys: pkcs8_private_keys(&mut BufReader::new(File::open(keys_path)?))
            .next()
            .ok_or(io::ErrorKind::InvalidInput)?
            .map(Into::into)?,
    }))
}

impl SingleCerts {
    pub fn new() -> io::Result<Self> {
        if let Some(certs) = load_from_env()? {
            info!("Certifications loaded from environment variables.");
            return Ok(certs);
        }
        if let Some((certs, certs_path, keys_path)) = load_from_env_path()? {
            info!("Certification loaded from {:?}", certs_path);
            info!("Private key loaded from {:?}", keys_path);
            return Ok(certs);
        }
        if let Some(certs) = load_from_file()? {
            info!("Certification loaded from ./certs/server.crt");
            info!("Private key loaded from ./certs/server.key");
            return Ok(certs);
        }
        Err(Error::from(ErrorKind::NotFound))
    }
    pub fn gen_server_config(self) -> io::Result<ServerConfig> {
        rustls::ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
            .with_no_client_auth()
            .with_single_cert(self.certs, self.keys)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
    }
}
