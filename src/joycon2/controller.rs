//! Joy-Con controller management
//!
//! This module handles the input processing and state management for the
//! Joy-Con controllers, including button mapping and stick input.

use crate::joycon2::types::{Accelerometer, Buttons, Gyroscope, Stick};

/// Orientation of the controller
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Vertical/upright orientation (default)
    Vertical = 0,
    /// Horizontal/sideways orientation
    Horizontal = 1,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Vertical
    }
}

/// Joy-Con 2 Left controller state
#[derive(Debug, Clone)]
pub struct Joy2L {
    /// Controller name
    pub name: String,
    
    /// Controller side
    pub side: String,
    
    /// Controller orientation
    pub orientation: Orientation,
    
    /// MAC address
    pub mac_address: String,
    
    /// Button states (mapped for upright usage)
    pub buttons: LeftButtons,
    
    /// Analog stick (mapped for upright usage)
    pub analog_stick: Stick,
    
    /// Accelerometer data
    pub accelerometer: Accelerometer,
    
    /// Gyroscope data
    pub gyroscope: Gyroscope,
    
    /// Mouse position (from Joy-Con 2 trackpad/sensor)
    pub mouse: MouseData,
    
    /// Mouse button states
    pub mouse_btn: MouseButtons,
    
    /// Timestamp from controller
    pub timestamp: u32,
    
    /// Motion timestamp
    pub motion_timestamp: i32,
    
    /// Battery level (0.0 to 100.0)
    pub battery_level: f32,
    
    /// Low battery alert sent flag
    pub alert_sent: bool,
    
    /// Connection status
    pub is_connected: bool,
}

/// Left Joy-Con specific buttons
#[derive(Debug, Clone, Default)]
pub struct LeftButtons {
    pub zl: bool,
    pub l: bool,
    pub minus: bool,
    pub sll: bool,  // Side Left (SR on left controller)
    pub srl: bool,  // Side Right (SL on left controller)
    pub left: bool,
    pub down: bool,
    pub up: bool,
    pub right: bool,
    pub l3: bool,   // Left stick click
    pub capture: bool,
}

/// Mouse data from Joy-Con 2
#[derive(Debug, Clone, Default)]
pub struct MouseData {
    pub x: i16,
    pub y: i16,
    pub distance: u8,
}

/// Mouse button states
#[derive(Debug, Clone, Default)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub scroll_x: i16,
    pub scroll_y: i16,
}

/// Stick calibration values
#[derive(Debug, Clone, Copy)]
pub struct StickCalibration {
    pub x_min: u16,
    pub x_max: u16,
    pub y_min: u16,
    pub y_max: u16,
}

impl Default for StickCalibration {
    fn default() -> Self {
        // Default calibration values from Python code
        Self {
            x_min: 780,
            x_max: 3260,
            y_min: 820,
            y_max: 3250,
        }
    }
}

impl Default for Joy2L {
    fn default() -> Self {
        Self {
            name: "Joy-Con".to_string(),
            side: "Left".to_string(),
            orientation: Orientation::default(),
            mac_address: String::new(),
            buttons: LeftButtons::default(),
            analog_stick: Stick::default(),
            accelerometer: Accelerometer::default(),
            gyroscope: Gyroscope::default(),
            mouse: MouseData::default(),
            mouse_btn: MouseButtons::default(),
            timestamp: 0,
            motion_timestamp: 0,
            battery_level: 100.0,
            alert_sent: false,
            is_connected: false,
        }
    }
}

impl Joy2L {
    /// Create a new Joy-Con 2 Left controller
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the MAC address
    pub fn set_mac_address(&mut self, mac_address: String) {
        self.mac_address = mac_address;
    }
    
    /// Update controller state from BLE data
    pub fn update(&mut self, data: &[u8]) {
        self.parse_input_report(data);
    }
    
    /// Parse input report data
    fn parse_input_report(&mut self, data: &[u8]) {
        if data.len() < 0x3C {
            return; // Not enough data
        }
        
        // Parse button data (bytes 5-6)
        let btn_data = ((data[5] as u16) << 8) | (data[6] as u16);
        
        // Parse joystick data (bytes 10-12)
        let joystick_data = &data[10..13];
        
        // Parse mouse data (bytes 16-23)
        if data.len() >= 24 {
            let mouse_data = &data[16..24];
            self.mouse.x = i16::from_le_bytes([mouse_data[0], mouse_data[1]]);
            self.mouse.y = i16::from_le_bytes([mouse_data[2], mouse_data[3]]);
            if mouse_data.len() >= 8 {
                self.mouse.distance = mouse_data[7];
            }
        }
        
        // Parse timestamp (bytes 0-3)
        self.timestamp = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        
        // Parse motion timestamp (bytes 0x2A-0x2D)
        if data.len() >= 0x2E {
            self.motion_timestamp = i32::from_le_bytes([
                data[0x2A], data[0x2B], data[0x2C], data[0x2D]
            ]);
        }
        
        // Parse accelerometer (bytes 0x30-0x35)
        if data.len() >= 0x36 {
            let accel_x_raw = i16::from_le_bytes([data[0x30], data[0x31]]);
            let accel_y_raw = i16::from_le_bytes([data[0x32], data[0x33]]);
            let accel_z_raw = i16::from_le_bytes([data[0x34], data[0x35]]);
            
            let accel_factor = 1.0 / 4096.0; // 1G = 4096
            
            self.accelerometer.x = -(accel_x_raw as f32) * accel_factor;
            self.accelerometer.y = -(accel_z_raw as f32) * accel_factor;
            self.accelerometer.z = (accel_y_raw as f32) * accel_factor;
        }
        
        // Parse gyroscope (bytes 0x36-0x3B)
        if data.len() >= 0x3C {
            let gyro_x_raw = i16::from_le_bytes([data[0x36], data[0x37]]);
            let gyro_y_raw = i16::from_le_bytes([data[0x38], data[0x39]]);
            let gyro_z_raw = i16::from_le_bytes([data[0x3A], data[0x3B]]);
            
            let gyro_factor = 360.0 / 6048.0; // 360° = 6048
            
            self.gyroscope.x = (gyro_x_raw as f32) * gyro_factor; // Pitch
            self.gyroscope.y = -(gyro_z_raw as f32) * gyro_factor; // Roll
            self.gyroscope.z = (gyro_y_raw as f32) * gyro_factor; // Yaw
        }
        
        // Parse button states
        self.buttons.sll = (btn_data & 0x0020) != 0;
        self.buttons.srl = (btn_data & 0x0010) != 0;
        self.buttons.minus = (btn_data & 0x0100) != 0;
        self.buttons.l = (btn_data & 0x0040) != 0;
        self.buttons.zl = (btn_data & 0x0080) != 0;
        self.buttons.left = (btn_data & 0x0008) != 0;
        self.buttons.down = (btn_data & 0x0001) != 0;
        self.buttons.up = (btn_data & 0x0002) != 0;
        self.buttons.right = (btn_data & 0x0004) != 0;
        self.buttons.l3 = (btn_data & 0x0800) != 0;
        self.buttons.capture = (btn_data & 0x2000) != 0;
        
        // Parse analog stick
        let (x, y) = Self::decode_joystick(joystick_data, self.orientation, &StickCalibration::default());
        self.analog_stick.x = x;
        self.analog_stick.y = y;
        
        // Parse mouse buttons (mapped from controller buttons)
        self.mouse_btn.left = self.buttons.l;  // L button
        self.mouse_btn.right = self.buttons.zl; // ZL button
        
        // Parse scroll from joystick
        let (scroll_x, scroll_y) = Self::decode_scroll(joystick_data, &StickCalibration::default());
        self.mouse_btn.scroll_x = scroll_x;
        self.mouse_btn.scroll_y = scroll_y;
        
        // Parse battery level (bytes 31-32)
        if data.len() >= 33 {
            let battery_raw = (data[31] as u16) | ((data[32] as u16) << 8);
            let new_battery = (battery_raw as f32 * 100.0 / 4095.0).round();
            
            // Only update if lower (or first reading)
            if new_battery < self.battery_level || !self.is_connected {
                self.battery_level = new_battery;
            }
            
            // Check for low battery
            if self.battery_level < 10.0 && self.is_connected && !self.alert_sent {
                self.notify_low_battery();
                self.alert_sent = true;
            }
        }
        
        self.is_connected = true;
    }
    
    /// Decode joystick data (returns normalized -1.0 to 1.0)
    fn decode_joystick(data: &[u8], orientation: Orientation, cal: &StickCalibration) -> (f32, f32) {
        if data.len() != 3 {
            return (0.0, 0.0);
        }
        
        // Decode 12-bit values
        let x_raw = ((data[1] as u16 & 0x0F) << 8) | (data[0] as u16);
        let y_raw = ((data[2] as u16) << 4) | ((data[1] as u16 & 0xF0) >> 4);
        
        // Normalize to 0.0-1.0
        let x_norm = ((x_raw.saturating_sub(cal.x_min) as f32) 
            / (cal.x_max - cal.x_min) as f32)
            .clamp(0.0, 1.0);
        
        let y_norm = 1.0 - ((y_raw.saturating_sub(cal.y_min) as f32) 
            / (cal.y_max - cal.y_min) as f32)
            .clamp(0.0, 1.0);
        
        // Convert to -1.0 to 1.0 range
        let mut x = x_norm * 2.0 - 1.0;
        let mut y = y_norm * 2.0 - 1.0;
        
        // Swap for horizontal orientation
        if orientation == Orientation::Horizontal {
            std::mem::swap(&mut x, &mut y);
        }
        
        (x, y)
    }
    
    /// Decode scroll values from joystick
    fn decode_scroll(data: &[u8], cal: &StickCalibration) -> (i16, i16) {
        if data.len() != 3 {
            return (0, 0);
        }
        
        let x_raw = ((data[1] as u16 & 0x0F) << 8) | (data[0] as u16);
        let y_raw = ((data[2] as u16) << 4) | ((data[1] as u16 & 0xF0) >> 4);
        
        // Center around zero
        let x_center = (cal.x_max + cal.x_min) as f32 / 2.0;
        let y_center = (cal.y_max + cal.y_min) as f32 / 2.0;
        
        let x = x_raw as f32 - x_center;
        let y = y_raw as f32 - y_center;
        
        // Normalize to [-32767, 32767]
        let x_range = (cal.x_max - cal.x_min) as f32 / 2.0;
        let y_range = (cal.y_max - cal.y_min) as f32 / 2.0;
        
        let mut x_scroll = ((x / x_range).clamp(-1.0, 1.0) * 32767.0) as i16;
        let mut y_scroll = ((y / y_range).clamp(-1.0, 1.0) * 32767.0) as i16;
        
        // Apply deadzone
        const SCROLL_DEADZONE: i16 = 3000;
        if x_scroll.abs() < SCROLL_DEADZONE {
            x_scroll = 0;
        }
        if y_scroll.abs() < SCROLL_DEADZONE {
            y_scroll = 0;
        }
        
        (x_scroll, y_scroll)
    }
    
    /// Notify user of low battery
    fn notify_low_battery(&self) {
        let msg = format!("{} {} : low battery ({:.0}%)", 
            self.name, self.side, self.battery_level);
        
        #[cfg(windows)]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use std::iter::once;
            
            let title: Vec<u16> = OsStr::new("Alert Joy-Con")
                .encode_wide()
                .chain(once(0))
                .collect();
            
            let message: Vec<u16> = OsStr::new(&msg)
                .encode_wide()
                .chain(once(0))
                .collect();
            
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, MB_ICONWARNING};
                let _ = MessageBoxW(
                    None,
                    windows::core::PCWSTR(message.as_ptr()),
                    windows::core::PCWSTR(title.as_ptr()),
                    MB_OK | MB_ICONWARNING,
                );
            }
        }
        
        #[cfg(not(windows))]
        {
            eprintln!("[Alert] {}", msg);
        }
    }
    
    /// Print controller status (for debugging)
    pub fn print_status(&self) {
        println!("JoyCon Left Status:");
        println!("  Buttons: ZL={}, L={}, Minus={}, Capture={}, L3={}", 
            self.buttons.zl, self.buttons.l, self.buttons.minus, 
            self.buttons.capture, self.buttons.l3);
        println!("  D-Pad: U={}, D={}, L={}, R={}", 
            self.buttons.up, self.buttons.down, 
            self.buttons.left, self.buttons.right);
        println!("  Analog Stick: X={:.2}, Y={:.2}", 
            self.analog_stick.x, self.analog_stick.y);
        println!("  Mouse: X={}, Y={}, Distance=0x{:02X}", 
            self.mouse.x, self.mouse.y, self.mouse.distance);
        println!("  Battery Level: {:.0}%", self.battery_level);
        println!("  Connected: {}", self.is_connected);
    }
    
    /// Convert to generic Buttons struct (for mapping)
    pub fn to_buttons(&self) -> Buttons {
        Buttons {
            // Map Joy-Con Left buttons to generic buttons
            a: false, // Not on left Joy-Con
            b: false,
            x: false,
            y: false,
            l: self.buttons.l,
            r: false,
            zl: self.buttons.zl,
            zr: false,
            plus: false,
            minus: self.buttons.minus,
            home: false,
            capture: self.buttons.capture,
            chat: false,  // Not on left Joy-Con
            left_stick_click: self.buttons.l3,
            right_stick_click: false,
            dpad_up: self.buttons.up,
            dpad_down: self.buttons.down,
            dpad_left: self.buttons.left,
            dpad_right: self.buttons.right,
        }
    }
}

// ============================================================================
// Joy-Con 2 Right Controller
// ============================================================================

/// Right Joy-Con specific buttons
#[derive(Debug, Clone, Default)]
pub struct RightButtons {
    pub zr: bool,
    pub r: bool,
    pub plus: bool,
    pub slr: bool,  // Side Left (SL on right controller)
    pub srr: bool,  // Side Right (SR on right controller)
    pub y: bool,
    pub b: bool,
    pub x: bool,
    pub a: bool,
    pub r3: bool,   // Right stick click
    pub home: bool,
    pub chat: bool,  // Chat button (Joy-Con 2 specific)
}

/// Joy-Con 2 Right controller state
#[derive(Debug, Clone)]
pub struct Joy2R {
    /// Controller name
    pub name: String,
    
    /// Controller side
    pub side: String,
    
    /// Controller orientation
    pub orientation: Orientation,
    
    /// MAC address
    pub mac_address: String,
    
    /// Button states (mapped for upright usage)
    pub buttons: RightButtons,
    
    /// Analog stick (mapped for upright usage)
    pub analog_stick: Stick,
    
    /// Accelerometer data
    pub accelerometer: Accelerometer,
    
    /// Gyroscope data
    pub gyroscope: Gyroscope,
    
    /// Mouse position (from Joy-Con 2 trackpad/sensor)
    pub mouse: MouseData,
    
    /// Mouse button states
    pub mouse_btn: MouseButtons,
    
    /// Timestamp from controller
    pub timestamp: u32,
    
    /// Motion timestamp
    pub motion_timestamp: i32,
    
    /// Battery level (0.0 to 100.0)
    pub battery_level: f32,
    
    /// Low battery alert sent flag
    pub alert_sent: bool,
    
    /// Connection status
    pub is_connected: bool,
}

impl Default for Joy2R {
    fn default() -> Self {
        Self {
            name: "Joy-Con".to_string(),
            side: "Right".to_string(),
            orientation: Orientation::default(),
            mac_address: String::new(),
            buttons: RightButtons::default(),
            analog_stick: Stick::default(),
            accelerometer: Accelerometer::default(),
            gyroscope: Gyroscope::default(),
            mouse: MouseData::default(),
            mouse_btn: MouseButtons::default(),
            timestamp: 0,
            motion_timestamp: 0,
            battery_level: 100.0,
            alert_sent: false,
            is_connected: false,
        }
    }
}

impl Joy2R {
    /// Create a new Joy-Con 2 Right controller
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the MAC address
    pub fn set_mac_address(&mut self, mac_address: String) {
        self.mac_address = mac_address;
    }
    
    /// Update controller state from BLE data
    pub fn update(&mut self, data: &[u8]) {
        self.parse_input_report(data);
    }
    
    /// Parse input report data
    fn parse_input_report(&mut self, data: &[u8]) {
        if data.len() < 0x3C {
            return; // Not enough data
        }
        
        // Parse button data (bytes 4-5 for right Joy-Con)
        let btn_data = ((data[4] as u16) << 8) | (data[5] as u16);
        
        // Parse joystick data (bytes 13-15 for right Joy-Con)
        let joystick_data = &data[13..16];
        
        // Parse mouse data (bytes 16-23)
        if data.len() >= 24 {
            let mouse_data = &data[16..24];
            self.mouse.x = i16::from_le_bytes([mouse_data[0], mouse_data[1]]);
            self.mouse.y = i16::from_le_bytes([mouse_data[2], mouse_data[3]]);
            if mouse_data.len() >= 8 {
                self.mouse.distance = mouse_data[7];
            }
        }
        
        // Parse timestamp (bytes 0-3)
        self.timestamp = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        
        // Parse motion timestamp (bytes 0x2A-0x2D)
        if data.len() >= 0x2E {
            self.motion_timestamp = i32::from_le_bytes([
                data[0x2A], data[0x2B], data[0x2C], data[0x2D]
            ]);
        }
        
        // Parse accelerometer (bytes 0x30-0x35)
        if data.len() >= 0x36 {
            let accel_x_raw = i16::from_le_bytes([data[0x30], data[0x31]]);
            let accel_y_raw = i16::from_le_bytes([data[0x32], data[0x33]]);
            let accel_z_raw = i16::from_le_bytes([data[0x34], data[0x35]]);
            
            let accel_factor = 1.0 / 4096.0; // 1G = 4096
            
            self.accelerometer.x = -(accel_x_raw as f32) * accel_factor;
            self.accelerometer.y = -(accel_z_raw as f32) * accel_factor;
            self.accelerometer.z = (accel_y_raw as f32) * accel_factor;
        }
        
        // Parse gyroscope (bytes 0x36-0x3B)
        if data.len() >= 0x3C {
            let gyro_x_raw = i16::from_le_bytes([data[0x36], data[0x37]]);
            let gyro_y_raw = i16::from_le_bytes([data[0x38], data[0x39]]);
            let gyro_z_raw = i16::from_le_bytes([data[0x3A], data[0x3B]]);
            
            let gyro_factor = 360.0 / 6048.0; // 360° = 6048
            
            self.gyroscope.x = (gyro_x_raw as f32) * gyro_factor; // Roll
            self.gyroscope.y = -(gyro_z_raw as f32) * gyro_factor; // Pitch
            self.gyroscope.z = (gyro_y_raw as f32) * gyro_factor; // Yaw
        }
        
        // Parse button states (different bitmask for right Joy-Con)
        self.buttons.zr = (btn_data & 0x8000) != 0;
        self.buttons.r = (btn_data & 0x4000) != 0;
        self.buttons.plus = (btn_data & 0x0002) != 0;
        self.buttons.slr = (btn_data & 0x2000) != 0;
        self.buttons.srr = (btn_data & 0x1000) != 0;
        self.buttons.y = (btn_data & 0x0100) != 0;
        self.buttons.b = (btn_data & 0x0400) != 0;
        self.buttons.x = (btn_data & 0x0200) != 0;
        self.buttons.a = (btn_data & 0x0800) != 0;
        self.buttons.r3 = (btn_data & 0x0004) != 0;
        self.buttons.home = (btn_data & 0x0010) != 0;
        self.buttons.chat = (btn_data & 0x0040) != 0;
        
        // Parse analog stick
        let (x, y) = Self::decode_joystick(joystick_data, self.orientation, &StickCalibration::default());
        self.analog_stick.x = x;
        self.analog_stick.y = y;
        
        // Parse mouse buttons (mapped from controller buttons)
        self.mouse_btn.left = self.buttons.r;  // R button
        self.mouse_btn.right = self.buttons.zr; // ZR button
        
        // Parse scroll from joystick
        let (scroll_x, scroll_y) = Self::decode_scroll(joystick_data, &StickCalibration::default());
        self.mouse_btn.scroll_x = scroll_x;
        self.mouse_btn.scroll_y = scroll_y;
        
        // Parse battery level (bytes 31-32)
        if data.len() >= 33 {
            let battery_raw = (data[31] as u16) | ((data[32] as u16) << 8);
            let new_battery = (battery_raw as f32 * 100.0 / 4095.0).round();
            
            // Only update if lower (or first reading)
            if new_battery < self.battery_level || !self.is_connected {
                self.battery_level = new_battery;
            }
            
            // Check for low battery
            if self.battery_level < 10.0 && self.is_connected && !self.alert_sent {
                self.notify_low_battery();
                self.alert_sent = true;
            }
        }
        
        self.is_connected = true;
    }
    
    /// Decode joystick data (returns normalized -1.0 to 1.0)
    fn decode_joystick(data: &[u8], orientation: Orientation, cal: &StickCalibration) -> (f32, f32) {
        if data.len() != 3 {
            return (0.0, 0.0);
        }
        
        // Decode 12-bit values
        let x_raw = ((data[1] as u16 & 0x0F) << 8) | (data[0] as u16);
        let y_raw = ((data[2] as u16) << 4) | ((data[1] as u16 & 0xF0) >> 4);
        
        // Normalize to 0.0-1.0
        let x_norm = ((x_raw.saturating_sub(cal.x_min) as f32) 
            / (cal.x_max - cal.x_min) as f32)
            .clamp(0.0, 1.0);
        
        let y_norm = 1.0 - ((y_raw.saturating_sub(cal.y_min) as f32) 
            / (cal.y_max - cal.y_min) as f32)
            .clamp(0.0, 1.0);
        
        // Convert to -1.0 to 1.0 range
        let mut x = x_norm * 2.0 - 1.0;
        let mut y = y_norm * 2.0 - 1.0;
        
        // Swap and invert for horizontal orientation (different for right Joy-Con)
        if orientation == Orientation::Horizontal {
            std::mem::swap(&mut x, &mut y);
            x = -x; // Invert X for horizontal on right Joy-Con
        }
        
        (x, y)
    }
    
    /// Decode scroll values from joystick
    fn decode_scroll(data: &[u8], cal: &StickCalibration) -> (i16, i16) {
        if data.len() != 3 {
            return (0, 0);
        }
        
        let x_raw = ((data[1] as u16 & 0x0F) << 8) | (data[0] as u16);
        let y_raw = ((data[2] as u16) << 4) | ((data[1] as u16 & 0xF0) >> 4);
        
        // Center around zero
        let x_center = (cal.x_max + cal.x_min) as f32 / 2.0;
        let y_center = (cal.y_max + cal.y_min) as f32 / 2.0;
        
        let x = x_raw as f32 - x_center;
        let y = y_raw as f32 - y_center;
        
        // Normalize to [-32767, 32767]
        let x_range = (cal.x_max - cal.x_min) as f32 / 2.0;
        let y_range = (cal.y_max - cal.y_min) as f32 / 2.0;
        
        let mut x_scroll = ((x / x_range).clamp(-1.0, 1.0) * 32767.0) as i16;
        let mut y_scroll = ((y / y_range).clamp(-1.0, 1.0) * 32767.0) as i16;
        
        // Apply deadzone
        const SCROLL_DEADZONE: i16 = 3000;
        if x_scroll.abs() < SCROLL_DEADZONE {
            x_scroll = 0;
        }
        if y_scroll.abs() < SCROLL_DEADZONE {
            y_scroll = 0;
        }
        
        (x_scroll, y_scroll)
    }
    
    /// Notify user of low battery
    fn notify_low_battery(&self) {
        let msg = format!("{} {} : low battery ({:.0}%)", 
            self.name, self.side, self.battery_level);
        
        #[cfg(windows)]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use std::iter::once;
            
            let title: Vec<u16> = OsStr::new("Alert Joy-Con")
                .encode_wide()
                .chain(once(0))
                .collect();
            
            let message: Vec<u16> = OsStr::new(&msg)
                .encode_wide()
                .chain(once(0))
                .collect();
            
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, MB_ICONWARNING};
                let _ = MessageBoxW(
                    None,
                    windows::core::PCWSTR(message.as_ptr()),
                    windows::core::PCWSTR(title.as_ptr()),
                    MB_OK | MB_ICONWARNING,
                );
            }
        }
        
        #[cfg(not(windows))]
        {
            eprintln!("[Alert] {}", msg);
        }
    }
    
    /// Print controller status (for debugging)
    pub fn print_status(&self) {
        println!("JoyCon Right Status:");
        println!("  Buttons: A={}, B={}, X={}, Y={}", 
            self.buttons.a, self.buttons.b, self.buttons.x, self.buttons.y);
        println!("  Shoulders: R={}, ZR={}, Plus={}, Home={}, Chat={}", 
            self.buttons.r, self.buttons.zr, self.buttons.plus,
            self.buttons.home, self.buttons.chat);
        println!("  Analog Stick: X={:.2}, Y={:.2}, R3={}", 
            self.analog_stick.x, self.analog_stick.y, self.buttons.r3);
        println!("  Mouse: X={}, Y={}, Distance=0x{:02X}", 
            self.mouse.x, self.mouse.y, self.mouse.distance);
        println!("  Battery Level: {:.0}%", self.battery_level);
        println!("  Connected: {}", self.is_connected);
    }
    
    /// Convert to generic Buttons struct (for mapping)
    pub fn to_buttons(&self) -> Buttons {
        Buttons {
            // Map Joy-Con Right buttons to generic buttons
            a: self.buttons.a,
            b: self.buttons.b,
            x: self.buttons.x,
            y: self.buttons.y,
            l: false,
            r: self.buttons.r,
            zl: false,
            zr: self.buttons.zr,
            plus: self.buttons.plus,
            minus: false,
            home: self.buttons.home,
            capture: false,
            chat: self.buttons.chat,
            left_stick_click: false,
            right_stick_click: self.buttons.r3,
            dpad_up: false,
            dpad_down: false,
            dpad_left: false,
            dpad_right: false,
        }
    }
}




