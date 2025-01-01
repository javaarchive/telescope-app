#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod states;
pub mod settings;
pub mod config;
pub mod oobe;
pub use app::TelescopeApp;