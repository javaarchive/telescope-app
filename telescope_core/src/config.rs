use std::{net::SocketAddr, path::PathBuf};

use log::error;
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CertificateAuthority {
    pub key_pair: String,
    pub certificate: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub ca: CertificateAuthority,
    pub addr: SocketAddr,
    pub data_dir: PathBuf,
    #[serde(skip)]
    // default to false
    #[serde(default)]
    pub loaded: bool
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ca: CertificateAuthority {
                key_pair: String::from("key_pair.pem"),
                certificate: String::from("certificate.pem")
            },
            addr: SocketAddr::from(([127, 0, 0, 1], 8080)),
            data_dir: std::env::current_dir().unwrap(),
            loaded: false
        }
    }
}

impl Config {
    pub fn try_load_or_default(data_dir: &PathBuf) -> Self {
        if data_dir.join("telescope_proxy.toml").exists() {
            // toml::from_str(&std::fs::read_to_string(data_dir.join("telescope_proxy.toml")).unwrap()).unwrap()
            match std::fs::read_to_string(data_dir.join("telescope_proxy.toml")) {
                Ok(config_str) => {
                    match toml::from_str::<Config>(&config_str) {
                        Ok(mut config) => {
                            config.loaded = true;
                            return config;
                        },
                        Err(e) => {
                            error!("Failed to parse proxy config file: {}", e);
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to read proxy config file: {}", e);
                }
            }
        }
        let mut config = Config::default();
        config.update_data_dir(data_dir.clone());
        config
    }

    pub fn update_data_dir(&mut self, data_dir: PathBuf) {
        self.data_dir = data_dir;
    }
}

impl Config {

}