//! Windows SendInput keyboard backend (scancode-based).
//!
//! This backend injects keyboard events using Win32 `SendInput` with
//! `KEYEVENTF_SCANCODE`, which is generally more reliable for games
//! than virtual-key based injection.
//!
//! # Supported Keys
//!
//! ## Letters
//! `a-z` (case-insensitive)
//!
//! ## Numbers
//! `0-9` (top row)
//!
//! ## Function Keys
//! `f1` through `f12`
//!
//! ## Modifiers
//! - `shift`, `leftshift`/`lshift`, `rightshift`/`rshift`
//! - `ctrl`/`control`, `leftctrl`/`lctrl`, `rightctrl`/`rctrl`
//! - `alt`, `leftalt`/`lalt`, `rightalt`/`ralt`
//!
//! ## Arrow Keys
//! `up`, `down`, `left`, `right` (also `uparrow`, `downarrow`, etc.)
//!
//! ## Numpad
//! `numpad0-9`/`kp0-9`, `numpadmultiply`/`kp*`, `numpadadd`/`kp+`,
//! `numpadsubtract`/`kp-`, `numpaddivide`/`kp/`, `numpaddecimal`/`kp.`,
//! `numpadenter`/`kpenter`
//!
//! ## Special Keys
//! `escape`/`esc`, `tab`, `capslock`/`caps`, `enter`/`return`,
//! `backspace`/`back`, `space`/`spacebar`, `insert`/`ins`, `delete`/`del`,
//! `home`, `end`, `pageup`/`pgup`, `pagedown`/`pgdown`
//!
//! ## Punctuation
//! `-`, `=`, `[`, `]`, `;`, `'`, `` ` ``, `\`, `,`, `.`, `/`
//!
//! # Safety Notes
//! - Calling `SendInput` is inherently unsafe; we wrap it in a small
//!   helper that returns a `windows::core::Result<()>` and surface a
//!   `Result<(), String>` at the public boundary.
//! - If `SendInput` returns 0, the last OS error is converted via
//!   `windows::core::Error::from_win32()`.
//! - Extended keys (arrows, numpad enter, right ctrl/alt, etc.) are
//!   automatically handled with the `KEYEVENTF_EXTENDEDKEY` flag.

#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VIRTUAL_KEY,
};

#[cfg(windows)]
/// Backend that uses Win32 SendInput to synthesize keyboard events.
#[derive(Clone, Copy, Debug)]
pub struct KeyboardSendInputBackend;

#[cfg(windows)]
/// Comprehensive set of keyboard keys for gaming.
///
/// Covers letters, numbers, function keys, modifiers, arrow keys,
/// numpad, and common control keys.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AllowedKey {
    // Letters A-Z
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Numbers 0-9 (top row)
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    
    // Function keys F1-F12
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Modifiers
    Shift, LeftShift, RightShift,
    Ctrl, LeftCtrl, RightCtrl,
    Alt, LeftAlt, RightAlt,
    
    // Arrow keys
    Up, Down, Left, Right,
    
    // Numpad
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadMultiply, NumpadAdd, NumpadSubtract,
    NumpadDivide, NumpadDecimal, NumpadEnter,
    
    // Special keys
    Escape, Tab, CapsLock, Enter, Backspace, Space,
    Insert, Delete, Home, End, PageUp, PageDown,
    
    // Punctuation and symbols
    Minus, Equals, LeftBracket, RightBracket,
    Semicolon, Apostrophe, Grave, Backslash,
    Comma, Period, Slash,
}

#[cfg(windows)]
impl AllowedKey {
    /// US keyboard Set 1 scancodes.
    /// Reference: https://www.win.tue.nl/~aeb/linux/kbd/scancodes-1.html
    #[inline]
    pub fn scancode(self) -> u16 {
        match self {
            // Letters A-Z (QWERTY layout positions)
            Self::A => 0x1E,
            Self::B => 0x30,
            Self::C => 0x2E,
            Self::D => 0x20,
            Self::E => 0x12,
            Self::F => 0x21,
            Self::G => 0x22,
            Self::H => 0x23,
            Self::I => 0x17,
            Self::J => 0x24,
            Self::K => 0x25,
            Self::L => 0x26,
            Self::M => 0x32,
            Self::N => 0x31,
            Self::O => 0x18,
            Self::P => 0x19,
            Self::Q => 0x10,
            Self::R => 0x13,
            Self::S => 0x1F,
            Self::T => 0x14,
            Self::U => 0x16,
            Self::V => 0x2F,
            Self::W => 0x11,
            Self::X => 0x2D,
            Self::Y => 0x15,
            Self::Z => 0x2C,
            
            // Numbers 0-9 (top row)
            Self::Key0 => 0x0B,
            Self::Key1 => 0x02,
            Self::Key2 => 0x03,
            Self::Key3 => 0x04,
            Self::Key4 => 0x05,
            Self::Key5 => 0x06,
            Self::Key6 => 0x07,
            Self::Key7 => 0x08,
            Self::Key8 => 0x09,
            Self::Key9 => 0x0A,
            
            // Function keys F1-F12
            Self::F1 => 0x3B,
            Self::F2 => 0x3C,
            Self::F3 => 0x3D,
            Self::F4 => 0x3E,
            Self::F5 => 0x3F,
            Self::F6 => 0x40,
            Self::F7 => 0x41,
            Self::F8 => 0x42,
            Self::F9 => 0x43,
            Self::F10 => 0x44,
            Self::F11 => 0x57,
            Self::F12 => 0x58,
            
            // Modifiers
            Self::Shift | Self::LeftShift => 0x2A,
            Self::RightShift => 0x36,
            Self::Ctrl | Self::LeftCtrl => 0x1D,
            Self::RightCtrl => 0xE01D, // Extended key
            Self::Alt | Self::LeftAlt => 0x38,
            Self::RightAlt => 0xE038, // Extended key
            
            // Arrow keys (extended keys)
            Self::Up => 0xE048,
            Self::Down => 0xE050,
            Self::Left => 0xE04B,
            Self::Right => 0xE04D,
            
            // Numpad
            Self::Numpad0 => 0x52,
            Self::Numpad1 => 0x4F,
            Self::Numpad2 => 0x50,
            Self::Numpad3 => 0x51,
            Self::Numpad4 => 0x4B,
            Self::Numpad5 => 0x4C,
            Self::Numpad6 => 0x4D,
            Self::Numpad7 => 0x47,
            Self::Numpad8 => 0x48,
            Self::Numpad9 => 0x49,
            Self::NumpadMultiply => 0x37,
            Self::NumpadAdd => 0x4E,
            Self::NumpadSubtract => 0x4A,
            Self::NumpadDivide => 0xE035,
            Self::NumpadDecimal => 0x53,
            Self::NumpadEnter => 0xE01C,
            
            // Special keys
            Self::Escape => 0x01,
            Self::Tab => 0x0F,
            Self::CapsLock => 0x3A,
            Self::Enter => 0x1C,
            Self::Backspace => 0x0E,
            Self::Space => 0x39,
            Self::Insert => 0xE052,
            Self::Delete => 0xE053,
            Self::Home => 0xE047,
            Self::End => 0xE04F,
            Self::PageUp => 0xE049,
            Self::PageDown => 0xE051,
            
            // Punctuation and symbols
            Self::Minus => 0x0C,        // -
            Self::Equals => 0x0D,       // =
            Self::LeftBracket => 0x1A,  // [
            Self::RightBracket => 0x1B, // ]
            Self::Semicolon => 0x27,    // ;
            Self::Apostrophe => 0x28,   // '
            Self::Grave => 0x29,        // `
            Self::Backslash => 0x2B,    // \
            Self::Comma => 0x33,        // ,
            Self::Period => 0x34,       // .
            Self::Slash => 0x35,        // /
        }
    }
    
    /// Check if this is an extended key (requires KEYEVENTF_EXTENDEDKEY flag).
    #[inline]
    pub fn is_extended(self) -> bool {
        self.scancode() > 0xFF
    }
}

#[cfg(windows)]
impl KeyboardSendInputBackend {

    /// Parse a key name into an AllowedKey (case-insensitive).
    #[inline]
    pub fn parse_allowed_key(name: &str) -> Result<AllowedKey, String> {
        let n = name.trim().to_ascii_lowercase();
        match n.as_str() {
            // Letters
            "a" => Ok(AllowedKey::A),
            "b" => Ok(AllowedKey::B),
            "c" => Ok(AllowedKey::C),
            "d" => Ok(AllowedKey::D),
            "e" => Ok(AllowedKey::E),
            "f" => Ok(AllowedKey::F),
            "g" => Ok(AllowedKey::G),
            "h" => Ok(AllowedKey::H),
            "i" => Ok(AllowedKey::I),
            "j" => Ok(AllowedKey::J),
            "k" => Ok(AllowedKey::K),
            "l" => Ok(AllowedKey::L),
            "m" => Ok(AllowedKey::M),
            "n" => Ok(AllowedKey::N),
            "o" => Ok(AllowedKey::O),
            "p" => Ok(AllowedKey::P),
            "q" => Ok(AllowedKey::Q),
            "r" => Ok(AllowedKey::R),
            "s" => Ok(AllowedKey::S),
            "t" => Ok(AllowedKey::T),
            "u" => Ok(AllowedKey::U),
            "v" => Ok(AllowedKey::V),
            "w" => Ok(AllowedKey::W),
            "x" => Ok(AllowedKey::X),
            "y" => Ok(AllowedKey::Y),
            "z" => Ok(AllowedKey::Z),
            
            // Numbers
            "0" => Ok(AllowedKey::Key0),
            "1" => Ok(AllowedKey::Key1),
            "2" => Ok(AllowedKey::Key2),
            "3" => Ok(AllowedKey::Key3),
            "4" => Ok(AllowedKey::Key4),
            "5" => Ok(AllowedKey::Key5),
            "6" => Ok(AllowedKey::Key6),
            "7" => Ok(AllowedKey::Key7),
            "8" => Ok(AllowedKey::Key8),
            "9" => Ok(AllowedKey::Key9),
            
            // Function keys
            "f1" => Ok(AllowedKey::F1),
            "f2" => Ok(AllowedKey::F2),
            "f3" => Ok(AllowedKey::F3),
            "f4" => Ok(AllowedKey::F4),
            "f5" => Ok(AllowedKey::F5),
            "f6" => Ok(AllowedKey::F6),
            "f7" => Ok(AllowedKey::F7),
            "f8" => Ok(AllowedKey::F8),
            "f9" => Ok(AllowedKey::F9),
            "f10" => Ok(AllowedKey::F10),
            "f11" => Ok(AllowedKey::F11),
            "f12" => Ok(AllowedKey::F12),
            
            // Modifiers
            "shift" => Ok(AllowedKey::Shift),
            "leftshift" | "lshift" => Ok(AllowedKey::LeftShift),
            "rightshift" | "rshift" => Ok(AllowedKey::RightShift),
            "ctrl" | "control" => Ok(AllowedKey::Ctrl),
            "leftctrl" | "lctrl" | "leftcontrol" => Ok(AllowedKey::LeftCtrl),
            "rightctrl" | "rctrl" | "rightcontrol" => Ok(AllowedKey::RightCtrl),
            "alt" => Ok(AllowedKey::Alt),
            "leftalt" | "lalt" => Ok(AllowedKey::LeftAlt),
            "rightalt" | "ralt" => Ok(AllowedKey::RightAlt),
            
            // Arrow keys
            "up" | "uparrow" => Ok(AllowedKey::Up),
            "down" | "downarrow" => Ok(AllowedKey::Down),
            "left" | "leftarrow" => Ok(AllowedKey::Left),
            "right" | "rightarrow" => Ok(AllowedKey::Right),
            
            // Numpad
            "numpad0" | "kp0" => Ok(AllowedKey::Numpad0),
            "numpad1" | "kp1" => Ok(AllowedKey::Numpad1),
            "numpad2" | "kp2" => Ok(AllowedKey::Numpad2),
            "numpad3" | "kp3" => Ok(AllowedKey::Numpad3),
            "numpad4" | "kp4" => Ok(AllowedKey::Numpad4),
            "numpad5" | "kp5" => Ok(AllowedKey::Numpad5),
            "numpad6" | "kp6" => Ok(AllowedKey::Numpad6),
            "numpad7" | "kp7" => Ok(AllowedKey::Numpad7),
            "numpad8" | "kp8" => Ok(AllowedKey::Numpad8),
            "numpad9" | "kp9" => Ok(AllowedKey::Numpad9),
            "numpadmultiply" | "kpmultiply" | "kp*" => Ok(AllowedKey::NumpadMultiply),
            "numpadadd" | "kpadd" | "kp+" => Ok(AllowedKey::NumpadAdd),
            "numpadsubtract" | "kpsubtract" | "kp-" => Ok(AllowedKey::NumpadSubtract),
            "numpaddivide" | "kpdivide" | "kp/" => Ok(AllowedKey::NumpadDivide),
            "numpaddecimal" | "kpdecimal" | "kp." => Ok(AllowedKey::NumpadDecimal),
            "numpadenter" | "kpenter" => Ok(AllowedKey::NumpadEnter),
            
            // Special keys
            "escape" | "esc" => Ok(AllowedKey::Escape),
            "tab" => Ok(AllowedKey::Tab),
            "capslock" | "caps" => Ok(AllowedKey::CapsLock),
            "enter" | "return" => Ok(AllowedKey::Enter),
            "backspace" | "back" => Ok(AllowedKey::Backspace),
            "space" | "spacebar" => Ok(AllowedKey::Space),
            "insert" | "ins" => Ok(AllowedKey::Insert),
            "delete" | "del" => Ok(AllowedKey::Delete),
            "home" => Ok(AllowedKey::Home),
            "end" => Ok(AllowedKey::End),
            "pageup" | "pgup" => Ok(AllowedKey::PageUp),
            "pagedown" | "pgdown" => Ok(AllowedKey::PageDown),
            
            // Punctuation and symbols
            "minus" | "-" => Ok(AllowedKey::Minus),
            "equals" | "=" => Ok(AllowedKey::Equals),
            "leftbracket" | "[" => Ok(AllowedKey::LeftBracket),
            "rightbracket" | "]" => Ok(AllowedKey::RightBracket),
            "semicolon" | ";" => Ok(AllowedKey::Semicolon),
            "apostrophe" | "quote" | "'" => Ok(AllowedKey::Apostrophe),
            "grave" | "`" => Ok(AllowedKey::Grave),
            "backslash" | "\\" => Ok(AllowedKey::Backslash),
            "comma" | "," => Ok(AllowedKey::Comma),
            "period" | "." => Ok(AllowedKey::Period),
            "slash" | "/" => Ok(AllowedKey::Slash),
            
            _ => Err(format!("unsupported key: '{name}'")),
        }
    }

    /// Press a key by name (w, a, s, d, shift).
    /// This is idempotent: repeated calls are safe but unnecessary for Hold.
    pub fn key_down(name: &str) -> Result<(), String> {
        let key = Self::parse_allowed_key(name)?;
        Self::key_down_scancode(key.scancode())
    }

    /// Release a key by name (w, a, s, d, shift).
    /// This is idempotent: repeated calls are safe but unnecessary for Hold.
    pub fn key_up(name: &str) -> Result<(), String> {
        let key = Self::parse_allowed_key(name)?;
        Self::key_up_scancode(key.scancode())
    }

    /// Low-level helper to send a single keyboard input using a hardware scancode.
    ///
    /// Flags should include `KEYEVENTF_SCANCODE` and optionally `KEYEVENTF_KEYUP`.
    /// For extended keys (scancode > 0xFF), the actual scancode is the lower byte
    /// and KEYEVENTF_EXTENDEDKEY flag is automatically added.
    unsafe fn send_scancode(scancode: u16, mut flags: KEYBD_EVENT_FLAGS) -> windows::core::Result<()> {
        use windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_EXTENDEDKEY;
        
        // Extract actual scancode and check if extended
        let actual_scancode = if scancode > 0xFF {
            // Extended key - add the extended flag
            flags |= KEYEVENTF_EXTENDEDKEY;
            (scancode & 0xFF) as u16
        } else {
            scancode
        };
        
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: actual_scancode,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        // Newer windows-rs supports passing a slice; keep this style for ergonomics.
        let sent = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
        if sent == 0 {
            use windows::Win32::Foundation::GetLastError;
            let err = unsafe { GetLastError() };
            Err(windows::core::Error::from_hresult(err.to_hresult()))
        } else {
            Ok(())
        }
    }

    /// Press a key by hardware scancode (make code).
    /// Press a key using a hardware scancode.
    pub fn key_down_scancode(scancode: u16) -> Result<(), String> {
        // SAFETY: Delegates to a thin wrapper around SendInput; caller ensures scancode validity.
        unsafe { Self::send_scancode(scancode, KEYEVENTF_SCANCODE) }.map_err(|e| format!("{e}"))
    }
    /// Release a key by hardware scancode (break code).
    pub fn key_up_scancode(scancode: u16) -> Result<(), String> {
        unsafe { Self::send_scancode(scancode, KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP) }
            .map_err(|e| format!("{e}"))
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::{KeyboardSendInputBackend as KB, AllowedKey};

    #[test]
    fn parse_letters() {
        assert!(matches!(KB::parse_allowed_key("w").unwrap(), AllowedKey::W));
        assert!(matches!(KB::parse_allowed_key("A").unwrap(), AllowedKey::A));
        assert!(matches!(KB::parse_allowed_key("z").unwrap(), AllowedKey::Z));
    }

    #[test]
    fn parse_numbers() {
        assert!(matches!(KB::parse_allowed_key("0").unwrap(), AllowedKey::Key0));
        assert!(matches!(KB::parse_allowed_key("5").unwrap(), AllowedKey::Key5));
        assert!(matches!(KB::parse_allowed_key("9").unwrap(), AllowedKey::Key9));
    }

    #[test]
    fn parse_function_keys() {
        assert!(matches!(KB::parse_allowed_key("f1").unwrap(), AllowedKey::F1));
        assert!(matches!(KB::parse_allowed_key("F12").unwrap(), AllowedKey::F12));
    }

    #[test]
    fn parse_modifiers() {
        assert!(matches!(KB::parse_allowed_key("Shift").unwrap(), AllowedKey::Shift));
        assert!(matches!(KB::parse_allowed_key("ctrl").unwrap(), AllowedKey::Ctrl));
        assert!(matches!(KB::parse_allowed_key("ALT").unwrap(), AllowedKey::Alt));
        assert!(matches!(KB::parse_allowed_key("leftshift").unwrap(), AllowedKey::LeftShift));
        assert!(matches!(KB::parse_allowed_key("rctrl").unwrap(), AllowedKey::RightCtrl));
    }

    #[test]
    fn parse_arrows() {
        assert!(matches!(KB::parse_allowed_key("up").unwrap(), AllowedKey::Up));
        assert!(matches!(KB::parse_allowed_key("down").unwrap(), AllowedKey::Down));
        assert!(matches!(KB::parse_allowed_key("leftarrow").unwrap(), AllowedKey::Left));
        assert!(matches!(KB::parse_allowed_key("right").unwrap(), AllowedKey::Right));
    }

    #[test]
    fn parse_numpad() {
        assert!(matches!(KB::parse_allowed_key("numpad0").unwrap(), AllowedKey::Numpad0));
        assert!(matches!(KB::parse_allowed_key("kp5").unwrap(), AllowedKey::Numpad5));
        assert!(matches!(KB::parse_allowed_key("kp+").unwrap(), AllowedKey::NumpadAdd));
        assert!(matches!(KB::parse_allowed_key("numpadenter").unwrap(), AllowedKey::NumpadEnter));
    }

    #[test]
    fn parse_special() {
        assert!(matches!(KB::parse_allowed_key("space").unwrap(), AllowedKey::Space));
        assert!(matches!(KB::parse_allowed_key("esc").unwrap(), AllowedKey::Escape));
        assert!(matches!(KB::parse_allowed_key("enter").unwrap(), AllowedKey::Enter));
        assert!(matches!(KB::parse_allowed_key("tab").unwrap(), AllowedKey::Tab));
        assert!(matches!(KB::parse_allowed_key("backspace").unwrap(), AllowedKey::Backspace));
    }

    #[test]
    fn parse_punctuation() {
        assert!(matches!(KB::parse_allowed_key("-").unwrap(), AllowedKey::Minus));
        assert!(matches!(KB::parse_allowed_key("=").unwrap(), AllowedKey::Equals));
        assert!(matches!(KB::parse_allowed_key("[").unwrap(), AllowedKey::LeftBracket));
        assert!(matches!(KB::parse_allowed_key(";").unwrap(), AllowedKey::Semicolon));
        assert!(matches!(KB::parse_allowed_key("/").unwrap(), AllowedKey::Slash));
    }

    #[test]
    fn parse_invalid() {
        assert!(KB::parse_allowed_key("invalid_key").is_err());
        assert!(KB::parse_allowed_key("f13").is_err());
    }

    #[test]
    fn extended_keys() {
        // Arrow keys are extended
        assert!(AllowedKey::Up.is_extended());
        assert!(AllowedKey::Right.is_extended());
        assert!(AllowedKey::Home.is_extended());
        assert!(AllowedKey::PageUp.is_extended());
        
        // Regular keys are not extended
        assert!(!AllowedKey::W.is_extended());
        assert!(!AllowedKey::Space.is_extended());
        assert!(!AllowedKey::F1.is_extended());
    }

    #[test]
    fn scancodes() {
        // Verify some known scancodes
        assert_eq!(AllowedKey::A.scancode(), 0x1E);
        assert_eq!(AllowedKey::W.scancode(), 0x11);
        assert_eq!(AllowedKey::Space.scancode(), 0x39);
        assert_eq!(AllowedKey::Escape.scancode(), 0x01);
        assert_eq!(AllowedKey::Up.scancode(), 0xE048);
        assert_eq!(AllowedKey::RightCtrl.scancode(), 0xE01D);
    }
}

