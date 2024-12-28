use std::{net::SocketAddr, path::PathBuf};


pub struct CertificateAuthority {
    pub key_pair: String,
    pub certificate: String
}

pub struct Config {
    pub ca: CertificateAuthority,
    pub addr: SocketAddr,
    pub data_dir: PathBuf
}