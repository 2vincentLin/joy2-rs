//! Integration tests for mock backends

use joy2_rs::backend::{KeyboardBackend, MockKeyboardBackend, MockMouseBackend, MouseBackend, MouseButton};

#[test]
fn test_mock_keyboard_backend() {
    let backend = MockKeyboardBackend::new();
    
    // All operations should succeed and print to stdout
    assert!(backend.key_down("w").is_ok());
    assert!(backend.key_up("w").is_ok());
    assert!(backend.key_press("space").is_ok());
    
    // Mock accepts any key name (unlike real backend)
    assert!(backend.key_down("invalid_key").is_ok());
}

#[test]
fn test_mock_mouse_backend() {
    let backend = MockMouseBackend::new();
    
    // All operations should succeed and print to stdout
    assert!(backend.move_relative(10, -5).is_ok());
    assert!(backend.button_down(MouseButton::Left).is_ok());
    assert!(backend.button_up(MouseButton::Left).is_ok());
    assert!(backend.click(MouseButton::Right).is_ok());
}

#[test]
fn test_mock_backends_are_clone() {
    let kb1 = MockKeyboardBackend::new();
    let kb2 = kb1.clone();
    
    let mb1 = MockMouseBackend::new();
    let mb2 = mb1.clone();
    
    // Both should work
    assert!(kb1.key_down("a").is_ok());
    assert!(kb2.key_down("b").is_ok());
    assert!(mb1.move_relative(1, 1).is_ok());
    assert!(mb2.move_relative(2, 2).is_ok());
}
