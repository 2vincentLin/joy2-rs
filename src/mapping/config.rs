//! Configuration loader and validator
//!
//! Loads mapping configuration from TOML files in the configs/ directory.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use thiserror::Error;
use log::{info, debug, warn};

/// Button type enum (for event-driven mapping)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ButtonType {
    A, B, X, Y,
    L, R, ZL, ZR,
    Plus, Minus, Home, Capture, Chat,  // Chat button (Joy-Con 2 specific)
    LeftStickClick, RightStickClick,
    DpadUp, DpadDown, DpadLeft, DpadRight,
    // Side buttons (SL/SR)
    SLL, SRL,  // Left Joy-Con side buttons
    SLR, SRR,  // Right Joy-Con side buttons
    
}

/// Stick type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StickType {
    Left,
    Right,
}

/// Controller side enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControllerSide {
    Left,
    Right,
}

/// Simplified Joy-Con state for mapping (TODO: integrate with Joy2L/Joy2R)
#[derive(Debug, Clone, Default)]
pub struct JoyConState {
    // Placeholder - will be replaced with actual controller state
}

/// Joy-Con event types
#[derive(Debug, Clone)]
pub enum JoyConEvent {
    ButtonPressed(ButtonType),
    ButtonReleased(ButtonType),
    StickMoved { stick: StickType, x: f32, y: f32 },
    GyroUpdate { side: ControllerSide, x: f32, y: f32, z: f32 },
    StateUpdate(Box<JoyConState>),
    Connected { side: ControllerSide },
    Disconnected { side: ControllerSide },
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Failed to parse config file: {0}")]
    Parse(#[from] toml::de::Error),
    
    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General settings
    #[serde(default)]
    pub settings: Settings,
    
    /// Multiple profiles (renamed from layers)
    #[serde(default)]
    pub profiles: Vec<Profile>,
}

/// General settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Left stick deadzone (0.0 to 1.0)
    #[serde(default = "default_deadzone")]
    pub left_stick_deadzone: f32,
    
    /// Right stick deadzone (0.0 to 1.0)
    #[serde(default = "default_deadzone")]
    pub right_stick_deadzone: f32,
    
    /// Enable vibration/rumble
    #[serde(default = "default_true")]
    pub vibration_enabled: bool,
    
    /// Default active profile name
    #[serde(default = "default_profile_name")]
    pub default_profile: String,
    
    /// Array of sensitivity multipliers to cycle through
    #[serde(default = "default_sensitivity_factors")]
    pub sensitivity_factor: Vec<f32>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            left_stick_deadzone: default_deadzone(),
            right_stick_deadzone: default_deadzone(),
            vibration_enabled: true,
            default_profile: default_profile_name(),
            sensitivity_factor: default_sensitivity_factors(),
        }
    }
}

fn default_deadzone() -> f32 { 0.15 }
fn default_true() -> bool { true }
fn default_profile_name() -> String { "base".to_string() }
fn default_sensitivity_factors() -> Vec<f32> { vec![1.0, 2.0, 3.0] }

/// A profile represents a complete set of mappings (renamed from Layer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    
    #[serde(default)]
    pub description: String,
    
    #[serde(default)]
    pub buttons: HashMap<ButtonType, Vec<Action>>,
    
    #[serde(default)]
    pub sticks: StickMappings,
    
    /// Gyroscope settings per controller
    #[serde(default)]
    pub gyro: GyroSettings,
    
    /// Button overrides when RIGHT gyro mouse is active
    #[serde(default)]
    pub gyro_mouse_overrides_right: HashMap<ButtonType, Vec<Action>>,
    
    /// Button overrides when LEFT gyro mouse is active
    #[serde(default)]
    pub gyro_mouse_overrides_left: HashMap<ButtonType, Vec<Action>>,
}

/// Gyroscope settings for both controllers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GyroSettings {
    #[serde(default)]
    pub left: GyroMapping,
    
    #[serde(default)]
    pub right: GyroMapping,
}

/// Stick mappings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StickMappings {
    /// Left stick mapping
    pub left: Option<StickMapping>,
    
    /// Right stick mapping
    pub right: Option<StickMapping>,
}

/// Stick mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickMapping {
    /// Mapping mode
    pub mode: StickMode,
    
    /// Sensitivity multiplier
    #[serde(default = "default_sensitivity")]
    pub sensitivity: f32,
    
    /// For directional mode: key bindings
    #[serde(default)]
    pub directions: Option<DirectionalKeys>,
}

fn default_sensitivity() -> f32 { 1.0 }

/// Stick mapping modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StickMode {
    /// Map to mouse movement (relative)
    Mouse,
    
    /// Map to WASD/arrow keys (directional)
    Directional,
    
    /// Disabled
    Disabled,
}

/// Directional key bindings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalKeys {
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
}

/// Gyroscope mapping per controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GyroMapping {
    /// Enable gyro-to-mouse
    #[serde(default)]
    pub enabled: bool,
    
    /// Output target (only "mouse" supported for PC)
    #[serde(default = "default_gyro_output")]
    pub output: String,
    
    /// Sensitivity for X-axis (yaw)
    #[serde(default = "default_sensitivity")]
    pub sensitivity_x: f32,
    
    /// Sensitivity for Y-axis (pitch)
    #[serde(default = "default_sensitivity")]
    pub sensitivity_y: f32,
    
    /// Invert X-axis
    #[serde(default)]
    pub invert_x: bool,
    
    /// Invert Y-axis
    #[serde(default)]
    pub invert_y: bool,
}

impl Default for GyroMapping {
    fn default() -> Self {
        Self {
            enabled: false,
            output: default_gyro_output(),
            sensitivity_x: 1.0,
            sensitivity_y: 1.0,
            invert_x: false,
            invert_y: false,
        }
    }
}

fn default_gyro_output() -> String { "mouse".to_string() }

/// Action to perform when input is triggered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Action {
    /// Do nothing (explicit no-op)
    None { 
        #[serde(default, deserialize_with = "deserialize_optional_key")]
        key: Option<String> 
    },
    
    /// Hold a key while button is held
    KeyHold { 
        #[serde(deserialize_with = "deserialize_optional_key")]
        key: Option<String> 
    },
    
    /// Move mouse relatively
    MouseMove { dx: i32, dy: i32 },
    
    /// Click mouse button
    MouseClick { button: MouseButton },
    
    /// Cycle to the next profile
    #[serde(rename = "cycleprofiles")]
    CycleProfiles,
    
    /// Cycle through sensitivity levels
    #[serde(rename = "cyclesensitivity")]
    CycleSensitivity,
    
    /// Toggle gyro mouse for left controller
    #[serde(rename = "togglegyromousel")]
    ToggleGyroMouseL,
    
    /// Toggle gyro mouse for right controller
    #[serde(rename = "togglegyromouser")]
    ToggleGyroMouseR,
}

/// Custom deserializer to convert empty strings to None and warn
fn deserialize_optional_key<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        // Log warning about empty string
        warn!("Empty string found in config. Consider using {{ type = \"none\" }} instead.");
        Ok(None)
    } else {
        Ok(Some(s))
    }
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path_ref = path.as_ref();
        info!("Loading configuration from: {}", path_ref.display());
        
        let content = std::fs::read_to_string(path_ref)?;
        let config: Config = toml::from_str(&content)?;
        
        info!("✓ Config parsed successfully");
        debug!("  - Profiles: {}", config.profiles.len());
        debug!("  - Default profile: '{}'", config.settings.default_profile);
        debug!("  - Sensitivity levels: {:?}", config.settings.sensitivity_factor);
        
        config.validate()?;
        info!("✓ Config validation passed");
        
        Ok(config)
    }
    
    /// Load default configuration from configs/default.toml
    pub fn load_default() -> Result<Self, ConfigError> {
        Self::load("configs/default.toml")
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate deadzones
        if self.settings.left_stick_deadzone < 0.0 || self.settings.left_stick_deadzone > 1.0 {
            return Err(ConfigError::Invalid(
                "left_stick_deadzone must be between 0.0 and 1.0".into()
            ));
        }
        
        if self.settings.right_stick_deadzone < 0.0 || self.settings.right_stick_deadzone > 1.0 {
            return Err(ConfigError::Invalid(
                "right_stick_deadzone must be between 0.0 and 1.0".into()
            ));
        }
        
        // Validate sensitivity factors
        for factor in &self.settings.sensitivity_factor {
            if *factor <= 0.0 {
                return Err(ConfigError::Invalid(
                    "sensitivity_factor values must be positive".into()
                ));
            }
        }
        
        // Validate profiles
        if self.profiles.is_empty() {
            return Err(ConfigError::Invalid(
                "At least one profile is required".into()
            ));
        }
        
        // Check if default profile exists
        let default_exists = self.profiles.iter()
            .any(|p| p.name == self.settings.default_profile);
        
        if !default_exists {
            return Err(ConfigError::Invalid(
                format!("Default profile '{}' not found", self.settings.default_profile)
            ));
        }
        
        // Validate each profile
        for profile in &self.profiles {
            self.validate_profile(profile)?;
        }
        
        // Validate toggle/cycle buttons are consistent across profiles
        self.validate_profile_switching_buttons()?;
        
        Ok(())
    }
    
    /// Validate a single profile's actions and key names
    fn validate_profile(&self, profile: &Profile) -> Result<(), ConfigError> {
        // Validate button actions
        for (button, actions) in &profile.buttons {
            for action in actions {
                self.validate_action(action, &format!("profile '{}' button {:?}", profile.name, button))?;
            }
        }
        
        // Validate gyro mouse override actions
        for (button, actions) in &profile.gyro_mouse_overrides_left {
            for action in actions {
                self.validate_action(action, &format!("profile '{}' gyro_mouse_overrides_left button {:?}", profile.name, button))?;
            }
        }
        
        for (button, actions) in &profile.gyro_mouse_overrides_right {
            for action in actions {
                self.validate_action(action, &format!("profile '{}' gyro_mouse_overrides_right button {:?}", profile.name, button))?;
            }
        }
        
        // Validate directional keys if present
        if let Some(ref left_stick) = profile.sticks.left {
            if let Some(ref dirs) = left_stick.directions {
                self.validate_key(&dirs.up, &format!("profile '{}' left stick up", profile.name))?;
                self.validate_key(&dirs.down, &format!("profile '{}' left stick down", profile.name))?;
                self.validate_key(&dirs.left, &format!("profile '{}' left stick left", profile.name))?;
                self.validate_key(&dirs.right, &format!("profile '{}' left stick right", profile.name))?;
            }
        }
        
        if let Some(ref right_stick) = profile.sticks.right {
            if let Some(ref dirs) = right_stick.directions {
                self.validate_key(&dirs.up, &format!("profile '{}' right stick up", profile.name))?;
                self.validate_key(&dirs.down, &format!("profile '{}' right stick down", profile.name))?;
                self.validate_key(&dirs.left, &format!("profile '{}' right stick left", profile.name))?;
                self.validate_key(&dirs.right, &format!("profile '{}' right stick right", profile.name))?;
            }
        }
        
        Ok(())
    }
    
    /// Validate a single action
    fn validate_action(&self, action: &Action, context: &str) -> Result<(), ConfigError> {
        match action {
            Action::KeyHold { key } | Action::None { key } => {
                if let Some(key_name) = key {
                    self.validate_key(key_name, context)?;
                }
            }
            Action::MouseMove { .. } | Action::MouseClick { .. } => {
                // Always valid
            }
            Action::CycleProfiles | Action::CycleSensitivity | 
            Action::ToggleGyroMouseL | Action::ToggleGyroMouseR => {
                // Validated separately in validate_profile_switching_buttons
            }
        }
        Ok(())
    }
    
    /// Validate a key name against the allowed keyboard backend keys
    fn validate_key(&self, key: &str, context: &str) -> Result<(), ConfigError> {
        // Check if it contains multi-key combo (e.g., "shift+w")
        if key.contains('+') {
            // Validate each part of the combo
            for part in key.split('+') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    self.validate_single_key(trimmed, context)?;
                }
            }
            Ok(())
        } else {
            self.validate_single_key(key, context)
        }
    }
    
    /// Validate a single key (not a combo)
    #[cfg(windows)]
    fn validate_single_key(&self, key: &str, context: &str) -> Result<(), ConfigError> {
        use crate::backend::keyboard_sendinput::KeyboardSendInputBackend;
        
        if let Err(_) = KeyboardSendInputBackend::parse_allowed_key(key) {
            return Err(ConfigError::Invalid(
                format!("Invalid key '{}' in {}: not supported by keyboard backend", key, context)
            ));
        }
        Ok(())
    }
    
    /// For non-Windows platforms, accept any key for now
    #[cfg(not(windows))]
    fn validate_single_key(&self, _key: &str, _context: &str) -> Result<(), ConfigError> {
        Ok(())
    }
    
    /// Validate that toggle/cycle action buttons are consistent across profiles
    /// This ensures users can always switch back from any profile
    fn validate_profile_switching_buttons(&self) -> Result<(), ConfigError> {
        if self.profiles.len() < 2 {
            // Only one profile, no need to validate switching
            return Ok(());
        }
        
        // Collect all buttons that have profile-switching actions
        let mut cycle_profile_buttons: HashSet<ButtonType> = HashSet::new();
        let mut toggle_gyro_l_buttons: HashSet<ButtonType> = HashSet::new();
        let mut toggle_gyro_r_buttons: HashSet<ButtonType> = HashSet::new();
        
        for profile in &self.profiles {
            // Check regular buttons
            for (button, actions) in &profile.buttons {
                for action in actions {
                    match action {
                        Action::CycleProfiles => {
                            cycle_profile_buttons.insert(*button);
                        }
                        Action::ToggleGyroMouseL => {
                            toggle_gyro_l_buttons.insert(*button);
                        }
                        Action::ToggleGyroMouseR => {
                            toggle_gyro_r_buttons.insert(*button);
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Now verify that ALL profiles have these buttons mapped to the same actions
        for profile in &self.profiles {
            // Check CycleProfiles consistency
            for button in &cycle_profile_buttons {
                let has_cycle = profile.buttons.get(button)
                    .map(|actions| actions.iter().any(|a| matches!(a, Action::CycleProfiles)))
                    .unwrap_or(false);
                
                if !has_cycle {
                    return Err(ConfigError::Invalid(
                        format!(
                            "Profile '{}' is missing CycleProfiles action on button {:?}. \
                            All profiles must have the same profile-switching buttons to allow switching back.",
                            profile.name, button
                        )
                    ));
                }
            }
            
            // Check ToggleGyroMouseL consistency
            for button in &toggle_gyro_l_buttons {
                let has_toggle = profile.buttons.get(button)
                    .map(|actions| actions.iter().any(|a| matches!(a, Action::ToggleGyroMouseL)))
                    .unwrap_or(false);
                
                if !has_toggle {
                    return Err(ConfigError::Invalid(
                        format!(
                            "Profile '{}' is missing ToggleGyroMouseL action on button {:?}. \
                            All profiles must have the same toggle buttons for consistency.",
                            profile.name, button
                        )
                    ));
                }
            }
            
            // Check ToggleGyroMouseR consistency
            for button in &toggle_gyro_r_buttons {
                let has_toggle = profile.buttons.get(button)
                    .map(|actions| actions.iter().any(|a| matches!(a, Action::ToggleGyroMouseR)))
                    .unwrap_or(false);
                
                if !has_toggle {
                    return Err(ConfigError::Invalid(
                        format!(
                            "Profile '{}' is missing ToggleGyroMouseR action on button {:?}. \
                            All profiles must have the same toggle buttons for consistency.",
                            profile.name, button
                        )
                    ));
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.left_stick_deadzone, 0.15);
        assert_eq!(settings.right_stick_deadzone, 0.15);
        assert!(settings.vibration_enabled);
        assert_eq!(settings.default_profile, "base");
        assert_eq!(settings.sensitivity_factor, vec![1.0, 2.0, 3.0]);
    }
    
    #[test]
    fn test_valid_config_minimal() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile {
                    name: "base".to_string(),
                    description: "Base profile".to_string(),
                    buttons: HashMap::new(),
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                }
            ],
        };
        
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_deadzone() {
        let mut config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile {
                    name: "base".to_string(),
                    description: "".to_string(),
                    buttons: HashMap::new(),
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                }
            ],
        };
        
        config.settings.left_stick_deadzone = 1.5;
        assert!(config.validate().is_err());
        
        config.settings.left_stick_deadzone = -0.1;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_invalid_sensitivity_factor() {
        let mut config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile {
                    name: "base".to_string(),
                    description: "".to_string(),
                    buttons: HashMap::new(),
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                }
            ],
        };
        
        config.settings.sensitivity_factor = vec![1.0, 0.0, 2.0];
        assert!(config.validate().is_err());
        
        config.settings.sensitivity_factor = vec![1.0, -1.0, 2.0];
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_missing_default_profile() {
        let config = Config {
            settings: Settings {
                default_profile: "nonexistent".to_string(),
                ..Settings::default()
            },
            profiles: vec![
                Profile {
                    name: "base".to_string(),
                    description: "".to_string(),
                    buttons: HashMap::new(),
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                }
            ],
        };
        
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_no_profiles() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![],
        };
        
        assert!(config.validate().is_err());
    }
    
    #[test]
    #[cfg(windows)]
    fn test_valid_key_names() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile {
                    name: "base".to_string(),
                    description: "".to_string(),
                    buttons: {
                        let mut map = HashMap::new();
                        map.insert(ButtonType::A, vec![Action::KeyHold { key: Some("w".to_string()) }]);
                        map.insert(ButtonType::B, vec![Action::KeyHold { key: Some("space".to_string()) }]);
                        map.insert(ButtonType::X, vec![Action::KeyHold { key: Some("f1".to_string()) }]);
                        map
                    },
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                }
            ],
        };
        
        assert!(config.validate().is_ok());
    }
    
    #[test]
    #[cfg(windows)]
    fn test_invalid_key_names() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile {
                    name: "base".to_string(),
                    description: "".to_string(),
                    buttons: {
                        let mut map = HashMap::new();
                        map.insert(ButtonType::A, vec![Action::KeyHold { key: Some("invalid_key_xyz".to_string()) }]);
                        map
                    },
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                }
            ],
        };
        
        assert!(config.validate().is_err());
    }
    
    #[test]
    #[cfg(windows)]
    fn test_valid_multi_key_combo() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile {
                    name: "base".to_string(),
                    description: "".to_string(),
                    buttons: HashMap::new(),
                    sticks: StickMappings {
                        left: Some(StickMapping {
                            mode: StickMode::Directional,
                            sensitivity: 1.0,
                            directions: Some(DirectionalKeys {
                                up: "shift+w".to_string(),
                                down: "ctrl+s".to_string(),
                                left: "a".to_string(),
                                right: "d".to_string(),
                            }),
                        }),
                        right: None,
                    },
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                }
            ],
        };
        
        assert!(config.validate().is_ok());
    }
    
    #[test]
    #[cfg(windows)]
    fn test_invalid_multi_key_combo() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile {
                    name: "base".to_string(),
                    description: "".to_string(),
                    buttons: HashMap::new(),
                    sticks: StickMappings {
                        left: Some(StickMapping {
                            mode: StickMode::Directional,
                            sensitivity: 1.0,
                            directions: Some(DirectionalKeys {
                                up: "shift+invalid".to_string(),
                                down: "s".to_string(),
                                left: "a".to_string(),
                                right: "d".to_string(),
                            }),
                        }),
                        right: None,
                    },
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                }
            ],
        };
        
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_cycle_profiles_consistency_valid() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile {
                    name: "base".to_string(),
                    description: "".to_string(),
                    buttons: {
                        let mut map = HashMap::new();
                        map.insert(ButtonType::SLR, vec![Action::CycleProfiles]);
                        map
                    },
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                },
                Profile {
                    name: "game".to_string(),
                    description: "".to_string(),
                    buttons: {
                        let mut map = HashMap::new();
                        map.insert(ButtonType::SLR, vec![Action::CycleProfiles]);
                        map
                    },
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                }
            ],
        };
        
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_cycle_profiles_consistency_invalid() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile {
                    name: "base".to_string(),
                    description: "".to_string(),
                    buttons: {
                        let mut map = HashMap::new();
                        map.insert(ButtonType::SLR, vec![Action::CycleProfiles]);
                        map
                    },
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                },
                Profile {
                    name: "game".to_string(),
                    description: "".to_string(),
                    buttons: HashMap::new(), // Missing CycleProfiles!
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                }
            ],
        };
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing CycleProfiles"));
    }
    
    #[test]
    fn test_toggle_gyro_consistency_valid() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile {
                    name: "base".to_string(),
                    description: "".to_string(),
                    buttons: {
                        let mut map = HashMap::new();
                        map.insert(ButtonType::SRR, vec![Action::ToggleGyroMouseR]);
                        map
                    },
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                },
                Profile {
                    name: "game".to_string(),
                    description: "".to_string(),
                    buttons: {
                        let mut map = HashMap::new();
                        map.insert(ButtonType::SRR, vec![Action::ToggleGyroMouseR]);
                        map
                    },
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                }
            ],
        };
        
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_toggle_gyro_consistency_invalid() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile {
                    name: "base".to_string(),
                    description: "".to_string(),
                    buttons: {
                        let mut map = HashMap::new();
                        map.insert(ButtonType::SRR, vec![Action::ToggleGyroMouseR]);
                        map
                    },
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                },
                Profile {
                    name: "game".to_string(),
                    description: "".to_string(),
                    buttons: {
                        let mut map = HashMap::new();
                        // Different button for toggle - inconsistent!
                        map.insert(ButtonType::SLR, vec![Action::ToggleGyroMouseR]);
                        map
                    },
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                }
            ],
        };
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing ToggleGyroMouseR"));
    }
    
    #[test]
    fn test_action_none_with_key() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile {
                    name: "base".to_string(),
                    description: "".to_string(),
                    buttons: {
                        let mut map = HashMap::new();
                        map.insert(ButtonType::A, vec![Action::None { key: Some("w".to_string()) }]);
                        map
                    },
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                }
            ],
        };
        
        // None action with valid key should still validate the key
        #[cfg(windows)]
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_action_none_without_key() {
        let config = Config {
            settings: Settings::default(),
            profiles: vec![
                Profile {
                    name: "base".to_string(),
                    description: "".to_string(),
                    buttons: {
                        let mut map = HashMap::new();
                        map.insert(ButtonType::A, vec![Action::None { key: None }]);
                        map
                    },
                    sticks: StickMappings::default(),
                    gyro: GyroSettings::default(),
                    gyro_mouse_overrides_left: HashMap::new(),
                    gyro_mouse_overrides_right: HashMap::new(),
                }
            ],
        };
        
        assert!(config.validate().is_ok());
    }
}
