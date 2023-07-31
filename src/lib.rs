#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::KeloApp;

pub mod core;
pub mod editor;
pub mod renderer;
pub mod view;
