//! Test to verify mock backends log output correctly

use joy2_rs::backend::{KeyboardBackend, MockKeyboardBackend, MockMouseBackend, MouseBackend, MouseButton};

#[test]
fn test_mock_keyboard_logs() {
    // Initialize a simple logger for testing
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Info)
        .try_init();
    
    let backend = MockKeyboardBackend::new();
    
    // These should log at info level (visible with RUST_LOG=info)
    assert!(backend.key_down("w").is_ok());
    assert!(backend.key_up("w").is_ok());
    assert!(backend.key_press("space").is_ok());
}

#[test]
fn test_mock_mouse_logs() {
    // Initialize a simple logger for testing
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Info)
        .try_init();
    
    let backend = MockMouseBackend::new();
    
    // These should log at info level (visible with RUST_LOG=info)
    assert!(backend.move_relative(10, -5).is_ok());
    assert!(backend.button_down(MouseButton::Left).is_ok());
    assert!(backend.button_up(MouseButton::Left).is_ok());
    assert!(backend.click(MouseButton::Right).is_ok());
}
