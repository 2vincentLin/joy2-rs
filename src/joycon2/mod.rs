//! Joy-Con 2 controller support
//!
//! This module provides complete Joy-Con 2 integration including:
//! - BLE scanning and connection
//! - Protocol communication
//! - Input parsing and calibration
//! - Event-driven service

pub mod constants;
pub mod types;
pub mod controller;
pub mod connection;
pub mod mac_cache;

// Re-export commonly used items
pub use constants::*;
pub use types::*;
pub use controller::*;
pub use connection::*;
pub use mac_cache::*;

// TODO: Add these modules as we implement them
// pub mod protocol;
// pub mod parser;
// pub mod calibration;
// pub mod service;
