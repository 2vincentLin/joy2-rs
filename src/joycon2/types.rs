//! Joy-Con 2 type definitions
//!
//! This module defines the basic data types used throughout the joycon2 module,
//! including input states, sensor data, and button mappings.

use serde::{Deserialize, Serialize};

/// Analog stick state (normalized -1.0 to 1.0)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Stick {
    /// Horizontal axis (-1.0 = left, 1.0 = right)
    pub x: f32,
    
    /// Vertical axis (-1.0 = down, 1.0 = up)
    pub y: f32,
}

impl Default for Stick {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Gyroscope data (angular velocity in degrees per second)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Gyroscope {
    /// Roll rate (rotation around X-axis)
    pub x: f32,
    
    /// Pitch rate (rotation around Y-axis)
    pub y: f32,
    
    /// Yaw rate (rotation around Z-axis)
    pub z: f32,
}

impl Default for Gyroscope {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

/// Accelerometer data (linear acceleration in Gs)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Accelerometer {
    /// Acceleration along X-axis
    pub x: f32,
    
    /// Acceleration along Y-axis
    pub y: f32,
    
    /// Acceleration along Z-axis
    pub z: f32,
}

impl Default for Accelerometer {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

/// Generic button states
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Buttons {
    // Face buttons (right side)
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    
    // Shoulder buttons
    pub l: bool,
    pub r: bool,
    pub zl: bool,
    pub zr: bool,
    
    // System buttons
    pub plus: bool,
    pub minus: bool,
    pub home: bool,
    pub capture: bool,
    pub chat: bool,  // Chat button (Joy-Con 2 specific)
    
    // Stick clicks
    pub left_stick_click: bool,
    pub right_stick_click: bool,
    
    // D-pad
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
}
