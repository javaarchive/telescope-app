#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod utils;
pub mod states;
pub mod settings;
pub mod config;
pub mod oobe;
pub use app::TelescopeApp;
pub use app::AppState;