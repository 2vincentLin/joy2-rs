//! Mock mouse backend for testing.
//!
//! This backend logs mouse events instead of actually sending them
//! to the OS. Useful for testing the manager and mapping logic without
//! requiring actual input injection.

use log::info;

/// Mock mouse backend that logs events instead of sending them.
#[derive(Clone, Copy, Debug)]
pub struct MockMouseBackend;

impl MockMouseBackend {
    /// Create a new mock mouse backend.
    pub fn new() -> Self {
        Self
    }

    /// Move mouse relatively (logs to info level).
    pub fn move_relative(dx: i32, dy: i32) -> Result<(), String> {
        info!("[MOCK MOUSE] Move relative: dx={}, dy={}", dx, dy);
        Ok(())
    }

    /// Press a mouse button (logs to info level).
    pub fn button_down(button: &str) -> Result<(), String> {
        info!("[MOCK MOUSE] Button DOWN: {}", button);
        Ok(())
    }

    /// Release a mouse button (logs to info level).
    pub fn button_up(button: &str) -> Result<(), String> {
        info!("[MOCK MOUSE] Button UP: {}", button);
        Ok(())
    }

    /// Click a mouse button (logs to info level).
    pub fn button_click(button: &str) -> Result<(), String> {
        info!("[MOCK MOUSE] Button CLICK: {}", button);
        Ok(())
    }
}

impl Default for MockMouseBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::MockMouseBackend;

    #[test]
    fn mock_mouse_works() {
        // These should just print, not fail
        assert!(MockMouseBackend::move_relative(10, -5).is_ok());
        assert!(MockMouseBackend::button_down("left").is_ok());
        assert!(MockMouseBackend::button_up("left").is_ok());
        assert!(MockMouseBackend::button_click("right").is_ok());
        
        // Mock accepts any button name
        assert!(MockMouseBackend::button_down("invalid_button").is_ok());
    }
}
