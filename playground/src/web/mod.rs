//! Web interface for the Adze playground.
//!
//! This module is intentionally split by responsibility: DTO definitions,
//! session state wiring, static asset handlers, API handlers, and server setup.

mod assets;
mod dto;
mod handlers;
mod server;
mod state;

pub use server::launch_server;
