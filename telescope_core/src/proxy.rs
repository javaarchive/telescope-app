use std::{collections::HashMap, sync::{Arc, RwLock}};

use hudsucker::{certificate_authority::RcgenAuthority, decode_request, decode_response, hyper::{Request, Response}, rcgen::{self, CertificateParams, KeyPair}, rustls::crypto::aws_lc_rs, tokio_tungstenite::tungstenite::{http::request, Message}, Body, HttpContext, HttpHandler, Proxy, RequestOrResponse, WebSocketContext, WebSocketHandler};
use log::warn;
use tokio::sync::watch::Receiver;

use crate::{config::{self, Config}, resource::{Flow, FlowContent, HTTPPair, ResolveString}};

// rewrite
#[derive(Debug, Default)]
pub struct FlowStorage {
    pub flows: HashMap<String, Flow>,
    pub flow_id_timeline: Vec<String>,
}

impl FlowStorage {
    pub fn new() -> Self {
        Self {
            flows: HashMap::new(),
            flow_id_timeline: Vec::new()
        }
    }
    
    pub fn add_flow(&mut self, flow: Flow) {
        // 2 clones here
        let id = flow.get_id();
        self.flows.insert(id.clone(), flow);
        self.flow_id_timeline.push(id);
    }

    pub fn get_flow(&self, id: &str) -> Option<&Flow> {
        self.flows.get(id)
    }

    pub fn get_flow_mut(&mut self, id: &str) -> Option<&mut Flow> {
        self.flows.get_mut(id)
    }

    pub fn remove_flow(&mut self, id: &str) -> Option<Flow> {
        let flow_opt = self.flows.remove(id);
        if flow_opt.is_some() {
            self.flow_id_timeline.retain(|x| x != id);
        }
        flow_opt
    }

    pub fn iter_flow_timeline(&self) -> impl Iterator<Item=&Flow> {
        self.flow_id_timeline.iter().map(|id| self.flows.get(id).unwrap())
    }

    pub fn flow_by_index(&self, index: usize) -> Option<&Flow> {
        self.flow_id_timeline.get(index).and_then(|id| self.flows.get(id))
    }

    pub fn len(&self) -> usize {
        self.flows.len()
    }
}

pub struct TelescopeProxy {
    pub storage: Arc<RwLock<FlowStorage>>, // flow id -> flow
    pub config: Receiver<Config>, 
    // rwlock plugins
}

impl TelescopeProxy {
    pub fn new(config: Receiver<Config>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(FlowStorage::new())),
            config: config
        }
    }
}

#[derive(Clone)]
pub struct TelescopeProxyRef {
    pub proxy: Arc<RwLock<TelescopeProxy>>,
    pub config: Receiver<Config>,
}

#[derive(Debug)]
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
        // TODO: reorganize this
        let maybe_proxy = {

            let config = self.config.borrow();

            match KeyPair::from_pem(&config.resolve_string(&config.ca.key_pair)) {
                Ok(key_pair) => {
                    match CertificateParams::from_ca_cert_pem(&config.resolve_string(&config.ca.certificate)) {
                        Ok(ca_cert) => {
                            match ca_cert.self_signed(&key_pair) {
                                Ok(ca_cert) => {
                                    let ca = RcgenAuthority::new(key_pair, ca_cert, 1_000, aws_lc_rs::default_provider());
                                    match Proxy::builder()
                                        .with_addr(config.addr).with_ca(ca).with_rustls_client(aws_lc_rs::default_provider())
                                        .with_http_handler(TelescopeProxyHandler::new(self.clone()))
                                        .with_websocket_handler(TelescopeProxyHandler::new(self.clone()))
                                        .build() {
                                        Ok(proxy) => {
                                            Ok(proxy)
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
        };
        match maybe_proxy {
            Ok(proxy) => {
                match proxy.start().await {
                    Ok(_) => {
                        Ok(())
                    },
                    Err(e) => Err(StartupError::HudsuckerError(e))
                }
            },
            Err(e) => Err(e)
        }
    }
}

#[derive(Clone)]
pub struct TelescopeProxyHandler {
    pub proxy_ref: TelescopeProxyRef,
    pub flow_id: Option<String>,
    pub flow_storage: Arc<RwLock<FlowStorage>>,
}

impl TelescopeProxyHandler {
    pub fn new(proxy_ref: TelescopeProxyRef) -> Self {
        let flow_storage = {
            let proxy = proxy_ref.proxy.read().unwrap();
            proxy.storage.clone()
        };

        Self {
            proxy_ref,
            flow_id: None,
            flow_storage: flow_storage
        }
    }
}

impl WebSocketHandler for TelescopeProxyHandler {
    async fn handle_message(&mut self, _ctx: &WebSocketContext, msg: Message) -> Option<Message> {
        Some(msg)
    }
}

impl HttpHandler for TelescopeProxyHandler {
    async fn handle_request(&mut self, _ctx: &HttpContext, req: Request<Body> ) -> RequestOrResponse {

        // run plugins

        let should_track = true; // TODO: discard huge bodies
        if should_track {
            // let flow = Flow::new(FlowContent::RequestResponse(HTTPPair { request: RequestOrResponse::Request(req.), response: None })
            let (req_intermediate, duplicated_request) = crate::resource::RequestOrResponse::copy_request(req).await;

            let flow = Flow::new(FlowContent::RequestResponse(HTTPPair::new_request(req_intermediate)));
            self.flow_id = Some(flow.get_id());
            self.flow_storage.write().unwrap().add_flow(flow);

            return duplicated_request.into();
        }
        req.into()
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {

        // run plugins

        if let Some(flow_id) = &self.flow_id {
            // we are tracking this flow
            let (res_intermediate, duplicated_response) = crate::resource::RequestOrResponse::copy_response(res).await;
            // record into flow
            if let Some(flow) = self.flow_storage.write().unwrap().get_flow_mut(flow_id)  {
                match flow.content {
                    FlowContent::RequestResponse(ref mut http_pair) => {
                        http_pair.add_response(res_intermediate);
                    },
                }
            } else {
                warn!("flow id {} deleted, response not recorded", flow_id);
            }
            return duplicated_response;
        }
        res
    }
}