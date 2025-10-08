//! Windows SendInput mouse backend (relative movement).
//!
//! Minimal, focused backend to send relative mouse motion via Win32 SendInput.
//! Intended for steering/camera control in games that accept injected mouse deltas.
//! Keep higher-level timing and mapping in the scheduler/manager.
//!
//! Safety: Same caveats as keyboard backend; wraps SendInput and converts errors to String.

#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEINPUT, MOUSE_EVENT_FLAGS, 
    MOUSEEVENTF_MOVE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
};

#[cfg(windows)]
#[derive(Clone, Copy, Debug)]
pub struct MouseSendInputBackend;

#[cfg(windows)]
impl MouseSendInputBackend {
    /// Send a single relative mouse movement (dx, dy) in pixels.
    pub fn move_relative(dx: i32, dy: i32) -> Result<(), String> {
        // Build a MOUSEINPUT for relative movement
        let mi = MOUSEINPUT {
            dx,
            dy,
            mouseData: 0,
            dwFlags: MOUSE_EVENT_FLAGS(MOUSEEVENTF_MOVE.0),
            time: 0,
            dwExtraInfo: 0,
        };

        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 { mi },
        };

        // SAFETY: Win32 call; we pass a single INPUT struct slice.
        let sent = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
        if sent == 0 {
            use windows::Win32::Foundation::GetLastError;
            let err = unsafe { GetLastError() };
            Err(format!("SendInput failed: 0x{:08X}", err.0))
        } else {
            Ok(())
        }
    }

    /// Press a mouse button (button down event).
    pub fn button_down(button: &str) -> Result<(), String> {
        let flags = Self::parse_button_down_flag(button)?;
        Self::send_button_event(flags)
    }

    /// Release a mouse button (button up event).
    pub fn button_up(button: &str) -> Result<(), String> {
        let flags = Self::parse_button_up_flag(button)?;
        Self::send_button_event(flags)
    }

    /// Parse button name to down event flag.
    fn parse_button_down_flag(button: &str) -> Result<MOUSE_EVENT_FLAGS, String> {
        match button.trim().to_ascii_lowercase().as_str() {
            "left" | "l" | "mouse1" => Ok(MOUSEEVENTF_LEFTDOWN),
            "right" | "r" | "mouse2" => Ok(MOUSEEVENTF_RIGHTDOWN),
            "middle" | "m" | "mouse3" => Ok(MOUSEEVENTF_MIDDLEDOWN),
            _ => Err(format!("unsupported mouse button: '{button}' (allowed: left/l/mouse1, right/r/mouse2, middle/m/mouse3)")),
        }
    }

    /// Parse button name to up event flag.
    fn parse_button_up_flag(button: &str) -> Result<MOUSE_EVENT_FLAGS, String> {
        match button.trim().to_ascii_lowercase().as_str() {
            "left" | "l" | "mouse1" => Ok(MOUSEEVENTF_LEFTUP),
            "right" | "r" | "mouse2" => Ok(MOUSEEVENTF_RIGHTUP),
            "middle" | "m" | "mouse3" => Ok(MOUSEEVENTF_MIDDLEUP),
            _ => Err(format!("unsupported mouse button: '{button}' (allowed: left/l/mouse1, right/r/mouse2, middle/m/mouse3)")),
        }
    }

    /// Send a mouse button event.
    fn send_button_event(flags: MOUSE_EVENT_FLAGS) -> Result<(), String> {
        let mi = MOUSEINPUT {
            dx: 0,
            dy: 0,
            mouseData: 0,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
        };

        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 { mi },
        };

        // SAFETY: Win32 call; we pass a single INPUT struct slice.
        let sent = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
        if sent == 0 {
            use windows::Win32::Foundation::GetLastError;
            let err = unsafe { GetLastError() };
            Err(format!("SendInput failed: 0x{:08X}", err.0))
        } else {
            Ok(())
        }
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::MouseSendInputBackend as Mouse;

    #[test]
    fn simple_move_compiles() {
        // We won't actually assert system behavior in unit tests; just ensure call path compiles.
        let _ = Mouse::move_relative(50, 0);
    }

    #[test]
    fn button_parsing() {
        // Test button down parsing
        assert!(Mouse::parse_button_down_flag("left").is_ok());
        assert!(Mouse::parse_button_down_flag("L").is_ok());
        assert!(Mouse::parse_button_down_flag("mouse1").is_ok());
        assert!(Mouse::parse_button_down_flag("right").is_ok());
        assert!(Mouse::parse_button_down_flag("R").is_ok());
        assert!(Mouse::parse_button_down_flag("mouse2").is_ok());
        assert!(Mouse::parse_button_down_flag("middle").is_ok());
        assert!(Mouse::parse_button_down_flag("M").is_ok());
        assert!(Mouse::parse_button_down_flag("mouse3").is_ok());
        assert!(Mouse::parse_button_down_flag("invalid").is_err());

        // Test button up parsing
        assert!(Mouse::parse_button_up_flag("left").is_ok());
        assert!(Mouse::parse_button_up_flag("right").is_ok());
        assert!(Mouse::parse_button_up_flag("middle").is_ok());
        assert!(Mouse::parse_button_up_flag("invalid").is_err());
    }

    #[test]
    fn button_down_up_compiles() {
        // Just ensure the API compiles; we won't actually inject events in tests
        let _ = Mouse::button_down("left");
        let _ = Mouse::button_up("left");
        let _ = Mouse::button_down("right");
        let _ = Mouse::button_up("right");
        let _ = Mouse::button_down("middle");
        let _ = Mouse::button_up("middle");
    }
}
