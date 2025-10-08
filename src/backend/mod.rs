//! Backend abstraction for keyboard and mouse input injection
//!
//! This module provides a unified interface for sending keyboard and mouse
//! events to the operating system.

pub mod keyboard_sendinput;
pub mod mouse_sendinput;
pub mod mock_keyboard;
pub mod mock_mouse;

#[cfg(windows)]
pub use keyboard_sendinput::{KeyboardSendInputBackend, AllowedKey};
#[cfg(windows)]
pub use mouse_sendinput::MouseSendInputBackend;

pub use mock_keyboard::MockKeyboardBackend;
pub use mock_mouse::MockMouseBackend;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("Backend operation failed: {0}")]
    Operation(String),
    
    #[error("Unsupported key: {0}")]
    UnsupportedKey(String),
    
    #[error("Platform not supported")]
    PlatformNotSupported,
}

/// Unified backend interface for keyboard operations
pub trait KeyboardBackend {
    /// Press a key (key down event)
    fn key_down(&self, key: &str) -> Result<(), BackendError>;
    
    /// Release a key (key up event)
    fn key_up(&self, key: &str) -> Result<(), BackendError>;
    
    /// Press and release a key (complete key press)
    fn key_press(&self, key: &str) -> Result<(), BackendError> {
        self.key_down(key)?;
        self.key_up(key)?;
        Ok(())
    }
}

/// Unified backend interface for mouse operations
pub trait MouseBackend {
    /// Move mouse relatively by (dx, dy) pixels
    fn move_relative(&self, dx: i32, dy: i32) -> Result<(), BackendError>;
    
    /// Click a mouse button
    fn click(&self, button: MouseButton) -> Result<(), BackendError>;
    
    /// Press a mouse button (button down)
    fn button_down(&self, button: MouseButton) -> Result<(), BackendError>;
    
    /// Release a mouse button (button up)
    fn button_up(&self, button: MouseButton) -> Result<(), BackendError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

// Windows implementations
#[cfg(windows)]
impl KeyboardBackend for KeyboardSendInputBackend {
    fn key_down(&self, key: &str) -> Result<(), BackendError> {
        KeyboardSendInputBackend::key_down(key)
            .map_err(|e| BackendError::Operation(e))
    }
    
    fn key_up(&self, key: &str) -> Result<(), BackendError> {
        KeyboardSendInputBackend::key_up(key)
            .map_err(|e| BackendError::Operation(e))
    }
}

#[cfg(windows)]
impl MouseBackend for MouseSendInputBackend {
    fn move_relative(&self, dx: i32, dy: i32) -> Result<(), BackendError> {
        MouseSendInputBackend::move_relative(dx, dy)
            .map_err(|e| BackendError::Operation(e))
    }
    
    fn click(&self, button: MouseButton) -> Result<(), BackendError> {
        self.button_down(button)?;
        self.button_up(button)?;
        Ok(())
    }
    
    fn button_down(&self, button: MouseButton) -> Result<(), BackendError> {
        let button_str = match button {
            MouseButton::Left => "left",
            MouseButton::Right => "right",
            MouseButton::Middle => "middle",
        };
        MouseSendInputBackend::button_down(button_str)
            .map_err(|e| BackendError::Operation(e))
    }
    
    fn button_up(&self, button: MouseButton) -> Result<(), BackendError> {
        let button_str = match button {
            MouseButton::Left => "left",
            MouseButton::Right => "right",
            MouseButton::Middle => "middle",
        };
        MouseSendInputBackend::button_up(button_str)
            .map_err(|e| BackendError::Operation(e))
    }
}

// Mock backend implementations
impl KeyboardBackend for MockKeyboardBackend {
    fn key_down(&self, key: &str) -> Result<(), BackendError> {
        MockKeyboardBackend::key_down(key)
            .map_err(|e| BackendError::Operation(e))
    }
    
    fn key_up(&self, key: &str) -> Result<(), BackendError> {
        MockKeyboardBackend::key_up(key)
            .map_err(|e| BackendError::Operation(e))
    }
    
    fn key_press(&self, key: &str) -> Result<(), BackendError> {
        MockKeyboardBackend::key_press(key)
            .map_err(|e| BackendError::Operation(e))
    }
}

impl MouseBackend for MockMouseBackend {
    fn move_relative(&self, dx: i32, dy: i32) -> Result<(), BackendError> {
        MockMouseBackend::move_relative(dx, dy)
            .map_err(|e| BackendError::Operation(e))
    }
    
    fn click(&self, button: MouseButton) -> Result<(), BackendError> {
        let button_str = match button {
            MouseButton::Left => "left",
            MouseButton::Right => "right",
            MouseButton::Middle => "middle",
        };
        MockMouseBackend::button_click(button_str)
            .map_err(|e| BackendError::Operation(e))
    }
    
    fn button_down(&self, button: MouseButton) -> Result<(), BackendError> {
        let button_str = match button {
            MouseButton::Left => "left",
            MouseButton::Right => "right",
            MouseButton::Middle => "middle",
        };
        MockMouseBackend::button_down(button_str)
            .map_err(|e| BackendError::Operation(e))
    }
    
    fn button_up(&self, button: MouseButton) -> Result<(), BackendError> {
        let button_str = match button {
            MouseButton::Left => "left",
            MouseButton::Right => "right",
            MouseButton::Middle => "middle",
        };
        MockMouseBackend::button_up(button_str)
            .map_err(|e| BackendError::Operation(e))
    }
}

/// Get the default keyboard backend for the current platform
#[cfg(windows)]
pub fn get_keyboard_backend() -> impl KeyboardBackend {
    KeyboardSendInputBackend
}

/// Get the default mouse backend for the current platform
#[cfg(windows)]
pub fn get_mouse_backend() -> impl MouseBackend {
    MouseSendInputBackend
}

/// Get a mock keyboard backend for testing
pub fn get_mock_keyboard_backend() -> impl KeyboardBackend {
    MockKeyboardBackend
}

/// Get a mock mouse backend for testing
pub fn get_mock_mouse_backend() -> impl MouseBackend {
    MockMouseBackend
}

#[cfg(not(windows))]
pub fn get_keyboard_backend() -> Result<(), BackendError> {
    Err(BackendError::PlatformNotSupported)
}

#[cfg(not(windows))]
pub fn get_mouse_backend() -> Result<(), BackendError> {
    Err(BackendError::PlatformNotSupported)
}
