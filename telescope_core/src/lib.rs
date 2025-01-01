pub mod config;
pub mod certs;
pub mod resource;
pub mod proxy;

pub async fn run_standalone() {
    let config = config::Config::default();
    let (send, recv) = tokio::sync::watch::channel(config);
    let proxy = proxy::TelescopeProxy::new(recv);
}