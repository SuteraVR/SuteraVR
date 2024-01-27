use std::{fs::File, io::{self, BufReader}, path::Path};
use rustls_pemfile::{certs, rsa_private_keys};
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};


pub struct SingleCerts {
    pub certs: io::Result<Vec<CertificateDer<'static>>>,
    pub keys: io::Result<PrivateKeyDer<'static>>,
}

impl SingleCerts {
    pub fn new(certs_path: &Path, keys_path: &Path) -> io::Result<Self> {
        Ok(Self {
            certs: certs(&mut BufReader::new(File::open(certs_path)?)).collect(),
            keys: rsa_private_keys(&mut BufReader::new(File::open(keys_path)?))
                .next()
                .unwrap()
                .map(Into::into),
        })
    }
    
}
