use std::{collections::HashMap, sync::{Arc, RwLock}};

use hudsucker::{certificate_authority::RcgenAuthority, hyper::{Request, Response}, rcgen::{self, CertificateParams, KeyPair}, rustls::crypto::aws_lc_rs, tokio_tungstenite::tungstenite::Message, Body, HttpContext, HttpHandler, Proxy, RequestOrResponse, WebSocketContext, WebSocketHandler};
use tokio::sync::watch::Receiver;

use crate::{config::{self, Config}, resource::Flow};

pub struct TelescopeProxy {
    pub storage: HashMap<String, Flow>, // flow id -> flow
    pub flow_id_timeline: Vec<String>, // flow ids chronologically
    pub config: Receiver<Config>, 
}

impl TelescopeProxy {
    pub fn new(config: Receiver<Config>) -> Self {
        Self {
            storage: HashMap::new(),
            flow_id_timeline: Vec::new(),
            config: config
        }
    }

    pub fn iterate_flows_chronologically(&self) -> impl Iterator<Item=&Flow> {
        self.flow_id_timeline.iter().map(|id| self.storage.get(id).unwrap())
    }
}

#[derive(Clone)]
pub struct TelescopeProxyRef {
    pub proxy: Arc<RwLock<TelescopeProxy>>,
    pub config: Receiver<Config>,
}

pub enum StartupError {
    RcgenError(rcgen::Error),
    HudsuckerError(hudsucker::Error),
}

impl TelescopeProxyRef {
    pub fn wrap(proxy: TelescopeProxy) -> Self {
        Self {
            config: proxy.config.clone(),
            proxy: Arc::new(RwLock::new(proxy))
        }
    }

    pub async fn start(&self) -> Result<(), StartupError> {
        // start_panicable but actually handling errors
        // holy shit I need to learn how to not make these giant match chains
        match KeyPair::from_pem(&self.config.borrow().ca.key_pair) {
            Ok(key_pair) => {
                match CertificateParams::from_ca_cert_pem(&self.config.borrow().ca.certificate) {
                    Ok(ca_cert) => {
                        match ca_cert.self_signed(&key_pair) {
                            Ok(ca_cert) => {
                                let ca = RcgenAuthority::new(key_pair, ca_cert, 1_000, aws_lc_rs::default_provider());
                                match Proxy::builder().with_addr(self.config.borrow().addr).with_ca(ca).with_rustls_client(aws_lc_rs::default_provider()).build() {
                                    Ok(proxy) => {
                                        match proxy.start().await {
                                            Ok(_) => {
                                                Ok(())
                                            },
                                            Err(e) => Err(StartupError::HudsuckerError(e))
                                        }
                                    },
                                    Err(e) => Err(StartupError::HudsuckerError(e))
                                }
                            },
                            Err(e) => Err(StartupError::RcgenError(e))
                        }
                    },
                    Err(e) => Err(StartupError::RcgenError(e))
                }
            },
            Err(e) => Err(StartupError::RcgenError(e))
        }
    }

    #[deprecated]
    pub async fn start_panicable(&self) {
        // TODO: handle bad certs and keys
        let key_pair = KeyPair::from_pem(&self.config.borrow().ca.key_pair).expect("Failed to parse private key");
        let ca_cert = CertificateParams::from_ca_cert_pem(&self.config.borrow().ca.certificate)
            .expect("Failed to parse CA certificate")
            .self_signed(&key_pair)
            .expect("Failed to sign CA certificate");
    
        let ca = RcgenAuthority::new(key_pair, ca_cert, 1_000, aws_lc_rs::default_provider());

        let proxy = Proxy::builder().with_addr(self.config.borrow().addr).with_ca(ca).with_rustls_client(aws_lc_rs::default_provider()).build().expect("Proxy building failed.");
        proxy.start().await.expect("Proxy failed to start.");
    }
}

#[derive(Clone)]
pub struct TelescopeProxyHandler {
    pub proxy_ref: TelescopeProxyRef,
}

impl TelescopeProxyHandler {
    
}

impl WebSocketHandler for TelescopeProxyHandler {
    async fn handle_message(&mut self, _ctx: &WebSocketContext, msg: Message) -> Option<Message> {
        Some(msg)
    }
}

impl HttpHandler for TelescopeProxyHandler {
    async fn handle_request(&mut self, _ctx: &HttpContext, req: Request<Body> ) -> RequestOrResponse {
        req.into()
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
        res
    }
}