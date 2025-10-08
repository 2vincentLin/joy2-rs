//! Mock keyboard backend for testing.
//!
//! This backend logs keyboard events instead of actually sending them
//! to the OS. Useful for testing the manager and mapping logic without
//! requiring actual input injection.

use log::info;

/// Mock keyboard backend that logs events instead of sending them.
#[derive(Clone, Copy, Debug)]
pub struct MockKeyboardBackend;

impl MockKeyboardBackend {
    /// Create a new mock keyboard backend.
    pub fn new() -> Self {
        Self
    }

    /// Press a key (logs to info level).
    pub fn key_down(key: &str) -> Result<(), String> {
        info!("[MOCK KEYBOARD] Key DOWN: {}", key);
        Ok(())
    }

    /// Release a key (logs to info level).
    pub fn key_up(key: &str) -> Result<(), String> {
        info!("[MOCK KEYBOARD] Key UP: {}", key);
        Ok(())
    }

    /// Press and release a key (logs to info level).
    pub fn key_press(key: &str) -> Result<(), String> {
        info!("[MOCK KEYBOARD] Key PRESS: {}", key);
        Ok(())
    }
}

impl Default for MockKeyboardBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::MockKeyboardBackend;

    #[test]
    fn mock_keyboard_works() {
        // These should just print, not fail
        assert!(MockKeyboardBackend::key_down("w").is_ok());
        assert!(MockKeyboardBackend::key_up("w").is_ok());
        assert!(MockKeyboardBackend::key_press("space").is_ok());
        
        // Mock accepts any key name
        assert!(MockKeyboardBackend::key_down("invalid_key").is_ok());
    }
}
