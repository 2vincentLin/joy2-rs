//! Joy2-rs: Joy-Con 2 to Keyboard/Mouse Bridge
//!
//! This library provides event-driven Joy-Con 2 controller support for Windows,
//! mapping controller inputs to keyboard and mouse actions.

pub mod backend;
pub mod joycon2;
pub mod mapping;
pub mod manager;

// Re-export commonly used items
pub use backend::{KeyboardBackend, MouseBackend};
pub use joycon2::{Joy2L, Joy2R, Buttons, Stick, Gyroscope, Accelerometer};
pub use manager::JoyConManager;
pub use mapping::{Config, MappingExecutor};
