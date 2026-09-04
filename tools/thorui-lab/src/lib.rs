#![forbid(unsafe_code)]

pub mod report;
pub mod statistics;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::start;
