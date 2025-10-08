//! Mapping module - converts Joy-Con inputs to keyboard/mouse actions

pub mod config;
pub mod executor;

pub use config::{Config, ConfigError};
pub use executor::MappingExecutor;
