//! Mapping executor - executes actions based on Joy-Con events
//!
//! This is the core of the event-driven architecture. It receives Joy-Con
//! events and executes the corresponding keyboard/mouse actions based on
//! the loaded configuration.

use crate::backend::{KeyboardBackend, MouseBackend, MouseButton};
use crate::mapping::config::{Action, Config, StickMode, ButtonType, StickType, JoyConState, JoyConEvent, ControllerSide};
use log::{debug, info, warn, trace};
use std::collections::{HashSet, HashMap};

/// Reference counts of sources keeping a key logically held
#[derive(Default, Debug, Clone, Copy)]
struct SourceCounts {
    button: u32,
    stick: u32,
}

impl SourceCounts {
    fn total(&self) -> u32 { self.button + self.stick }
    fn is_empty(&self) -> bool { self.total() == 0 }
}

#[derive(Clone, Copy, Debug)]
enum KeySource { Button, Stick }

/// Tracks which keys/buttons are currently held (logical and physical)
#[derive(Default)]
struct HeldState {
    /// Joy-Con buttons currently physically pressed (for deduping press events)
    buttons: HashSet<ButtonType>,
    /// Per-key logical source counts
    key_sources: HashMap<String, SourceCounts>,
    /// Keys we have actually sent key_down for (OS state)
    keys_down: HashSet<String>,
}

impl HeldState {
    /// Press a key (from a specific source), this method will track sources and only send key_down when first claimed
    fn press_key<Kb: KeyboardBackend>(&mut self, key: &str, source: KeySource, keyboard: &Kb) {
        if key.is_empty() { return; }
        let entry = self.key_sources.entry(key.to_string()).or_insert_with(SourceCounts::default);
        let before = entry.total();
        match source {
            KeySource::Button => {
                // Allow multiple different buttons to contribute (refcount)
                entry.button = entry.button.saturating_add(1);
            }
            KeySource::Stick => {
                // Stick is a single logical claimant per direction; make idempotent
                if entry.stick > 0 { return; }
                entry.stick = 1;
            }
        };
        if before == 0 {
            // First claimant -> send key_down
            if let Err(e) = keyboard.key_down(key) { warn!("Failed to press key '{}': {}", key, e); } else { trace!("key_down '{}' (source {:?})", key, source); self.keys_down.insert(key.to_string()); }
        } else {
            trace!("key '{}' additional claim {:?} -> counts b:{} s:{}", key, source, entry.button, entry.stick);
        }
    }

    /// Release a key (from a specific source), it'll only be released when all sources release it
    fn release_key<Kb: KeyboardBackend>(&mut self, key: &str, source: KeySource, keyboard: &Kb) {
        if key.is_empty() { return; }
        if let Some(entry) = self.key_sources.get_mut(key) {
            match source {
                KeySource::Button => { if entry.button > 0 { entry.button -= 1; } else { return; } },
                KeySource::Stick => { if entry.stick > 0 { entry.stick = 0; } else { return; } },
            };
            if entry.is_empty() {
                // Last claimant -> send key_up
                if self.keys_down.remove(key) {
                    if let Err(e) = keyboard.key_up(key) { warn!("Failed to release key '{}': {}", key, e); } else { trace!("key_up '{}' (source {:?})", key, source); }
                }
                self.key_sources.remove(key);
            } else {
                trace!("key '{}' partial release {:?} -> counts b:{} s:{}", key, source, entry.button, entry.stick);
            }
        } else {
            // Silent ignore to avoid startup spam
        }
    }

    fn clear_all<Kb: KeyboardBackend>(&mut self, keyboard: &Kb) {
        for key in self.keys_down.drain() {
            if let Err(e) = keyboard.key_up(&key) { warn!("Failed to release key '{}': {}", key, e); }
        }
        self.key_sources.clear();
        self.buttons.clear();
    }
}

/// Gyro mouse state per controller
#[derive(Default)]
struct GyroMouseState {
    left_enabled: bool,
    right_enabled: bool,
}

/// Current stick positions for continuous movement
#[derive(Default, Clone, Copy)]
struct StickState {
    x: f32,
    y: f32,
}

/// Executes mapping actions based on Joy-Con events
pub struct MappingExecutor<K, M>
where
    K: KeyboardBackend,
    M: MouseBackend,
{
    config: Config,
    keyboard: K,
    mouse: M,
    held_state: HeldState,
    previous_state: JoyConState,
    
    /// Current active profile index
    current_profile_index: usize,
    
    /// Current sensitivity factor index
    current_sensitivity_index: usize,
    
    /// Gyro mouse state
    gyro_mouse_state: GyroMouseState,
    
    /// Current stick positions (for continuous movement)
    left_stick: StickState,
    right_stick: StickState,
}

impl<K, M> MappingExecutor<K, M>
where
    K: KeyboardBackend,
    M: MouseBackend,
{
    /// Create a new mapping executor with the given configuration and backends
    pub fn new(config: Config, keyboard: K, mouse: M) -> Self {
        // Find default profile index
        let current_profile_index = config.profiles.iter()
            .position(|p| p.name == config.settings.default_profile)
            .unwrap_or(0);
        
        if !config.profiles.is_empty() {
            info!("Starting with profile: '{}'", config.profiles[current_profile_index].name);
        }
        
        Self {
            config,
            keyboard,
            mouse,
            held_state: HeldState::default(),
            previous_state: JoyConState::default(),
            current_profile_index,
            current_sensitivity_index: 0,
            gyro_mouse_state: GyroMouseState::default(),
            left_stick: StickState::default(),
            right_stick: StickState::default(),
        }
    }
    
    /// Get the current active profile
    fn current_profile(&self) -> Option<&crate::mapping::config::Profile> {
        self.config.profiles.get(self.current_profile_index)
    }
    
    /// Get current button mappings (with gyro mouse overrides if active)
    fn get_button_actions(&self, button: ButtonType, side: ControllerSide) -> Option<Vec<Action>> {
        let profile = self.current_profile()?;
        
        // Check if gyro mouse is active for this side
        let gyro_active = match side {
            ControllerSide::Left => self.gyro_mouse_state.left_enabled,
            ControllerSide::Right => self.gyro_mouse_state.right_enabled,
        };
        
        if gyro_active {
            // Try to get override for this specific side
            let overrides = match side {
                ControllerSide::Left => &profile.gyro_mouse_overrides_left,
                ControllerSide::Right => &profile.gyro_mouse_overrides_right,
            };
            
            if let Some(actions) = overrides.get(&button) {
                return Some(actions.clone());
            }
        }
        
        // Fall back to normal button mapping
        profile.buttons.get(&button).cloned()
    }
    
    /// Get current sensitivity factor
    fn get_sensitivity_factor(&self) -> f32 {
        self.config.settings.sensitivity_factor
            .get(self.current_sensitivity_index)
            .copied()
            .unwrap_or(1.0)
    }
    
    /// Process a Joy-Con event and execute corresponding actions
    pub fn process_event(&mut self, event: &JoyConEvent) {
        match event {
            JoyConEvent::ButtonPressed(button) => {
                self.on_button_pressed(*button);
            }
            
            JoyConEvent::ButtonReleased(button) => {
                self.on_button_released(*button);
            }
            
            JoyConEvent::StickMoved { stick, x, y } => {
                self.on_stick_moved(*stick, *x, *y);
            }
            
            JoyConEvent::GyroUpdate { side, x, y, z } => {
                self.on_gyro_update(*side, *x, *y, *z);
            }
            
            JoyConEvent::StateUpdate(state) => {
                self.on_state_update(state);
            }
            
            JoyConEvent::Connected { side } => {
                debug!("Controller {:?} connected", side);
            }
            
            JoyConEvent::Disconnected { side } => {
                debug!("Controller {:?} disconnected", side);
                self.release_all_held_keys();
            }
        }
    }
    
    /// Update continuous stick movements and held buttons (call this periodically in a timer)
    pub fn update_continuous_movements(&mut self) {
        // Apply movement for both sticks based on their current positions
        self.apply_stick_movement(StickType::Left);
        self.apply_stick_movement(StickType::Right);
        
        // Re-apply all held button actions to maintain continuous input
        // This is needed because Joy-Con 2 stops sending button events when held
        // and Windows needs repeated key_down calls for key repeat to work
        // for button in self.held_state.buttons.clone() {
        //     let side = Self::button_to_side(button);
        //     if let Some(actions) = self.get_button_actions(button, side) {
        //         for action in &actions {
        //             // Only re-apply KeyHold actions (not one-time actions like CycleProfiles)
        //             if matches!(action, Action::KeyHold { .. }) {
        //                 self.execute_action(action, true, side);
        //             }
        //         }
        //     }
        // }
    }
    
    /// Handle button press
    fn on_button_pressed(&mut self, button: ButtonType) {
        // Track if button was already pressed (to avoid repeating one-time actions)
        let was_already_pressed = !self.held_state.buttons.insert(button);
        
        // Determine which side this button is from
        let side = Self::button_to_side(button);
        
        // Get actions (with potential gyro mouse overrides)
        if let Some(actions) = self.get_button_actions(button, side) {
            for action in actions {
                // Only execute one-time actions on first press
                // KeyHold actions are handled ONLY by update_continuous_movements()
                match action {
                    Action::CycleProfiles | 
                    Action::CycleSensitivity |
                    Action::ToggleGyroMouseL |
                    Action::ToggleGyroMouseR => {
                        if !was_already_pressed {
                            self.execute_action(&action, true, side);
                        }
                    }
                    Action::KeyHold { .. } => {
                        // KeyHold actions are ONLY processed in update_continuous_movements()
                        // This ensures proper keyboard repeat behavior (initial delay + repeat)
                        // Do nothing here
                        log::debug!("KeyHold action triggered: {:?}", action);
                        if !was_already_pressed {
                            self.execute_action(&action, true, side);
                        }
                    }
                    _ => {
                        // Execute other actions (MouseClick)
                        self.execute_action(&action, true, side);
                    }
                }
            }
        }
    }
    
    /// Determine which controller side a button belongs to
    fn button_to_side(button: ButtonType) -> ControllerSide {
        match button {
            ButtonType::A | ButtonType::B | ButtonType::X | ButtonType::Y |
            ButtonType::R | ButtonType::ZR | ButtonType::Plus | ButtonType::Home |
            ButtonType::RightStickClick | ButtonType::SLR | ButtonType::SRR | ButtonType::Chat => {
                ControllerSide::Right
            }
            _ => ControllerSide::Left
        }
    }
    
    /// Handle button release
    fn on_button_released(&mut self, button: ButtonType) {
        if !self.held_state.buttons.remove(&button) {
            return; // Wasn't pressed
        }
        
        // Determine side
        let side = Self::button_to_side(button);
        
        if let Some(actions) = self.get_button_actions(button, side) {
            for action in actions {
                self.execute_action(&action, false, side);
            }
        }
    }
    
    /// Handle stick movement
    fn on_stick_moved(&mut self, stick: StickType, x: f32, y: f32) {
        // Store the stick position for continuous movement
        match stick {
            StickType::Left => {
                self.left_stick.x = x;
                self.left_stick.y = y;
            }
            StickType::Right => {
                self.right_stick.x = x;
                self.right_stick.y = y;
            }
        }
        
        // Apply the stick movement immediately
        self.apply_stick_movement(stick);
    }
    
    /// Apply stick movement based on current stick position
    fn apply_stick_movement(&mut self, stick: StickType) {
        let profile = match self.current_profile() {
            Some(p) => p,
            None => return,
        };
        
        let mapping = match stick {
            StickType::Left => profile.sticks.left.as_ref(),
            StickType::Right => profile.sticks.right.as_ref(),
        };
        
        let Some(mapping) = mapping else {
            return;
        };
        
        let deadzone = match stick {
            StickType::Left => self.config.settings.left_stick_deadzone,
            StickType::Right => self.config.settings.right_stick_deadzone,
        };
        
        // Get current stick position
        let (x, y) = match stick {
            StickType::Left => (self.left_stick.x, self.left_stick.y),
            StickType::Right => (self.right_stick.x, self.right_stick.y),
        };
        
        // Apply deadzone
        let magnitude = (x * x + y * y).sqrt();
        if magnitude < deadzone {
            // In deadzone - release any held directional keys
            if matches!(mapping.mode, StickMode::Directional) {
                self.release_directional_keys(stick);
            }
            return;
        }
        
        match mapping.mode {
            StickMode::Mouse => {
                // Map to mouse movement with sensitivity factor
                let sensitivity_factor = self.get_sensitivity_factor();
                let dx = (x * mapping.sensitivity * sensitivity_factor * 10.0) as i32;
                let dy = (y * mapping.sensitivity * sensitivity_factor * 10.0) as i32; // Don't invert Y - pushing up should move mouse up
                
                if dx != 0 || dy != 0 {
                    if let Err(e) = self.mouse.move_relative(dx, dy) {
                        warn!("Failed to move mouse: {}", e);
                    }
                }
            }
            
            StickMode::Directional => {
                // Map to directional keys (WASD or custom)
                if let Some(directions) = mapping.directions.as_ref().cloned() {
                    self.handle_directional_keys(x, y, &directions);
                }
            }
            
            StickMode::Disabled => {}
        }
    }
    
    /// Handle gyroscope update
    fn on_gyro_update(&mut self, side: ControllerSide, x: f32, y: f32, _z: f32) {
        let profile = match self.current_profile() {
            Some(p) => p,
            None => return,
        };
        
        // Check if gyro mouse is enabled for this side
        let gyro_mouse_active = match side {
            ControllerSide::Left => self.gyro_mouse_state.left_enabled,
            ControllerSide::Right => self.gyro_mouse_state.right_enabled,
        };
        
        if !gyro_mouse_active {
            return;
        }
        
        // Get gyro settings for this side
        let gyro_settings = match side {
            ControllerSide::Left => &profile.gyro.left,
            ControllerSide::Right => &profile.gyro.right,
        };
        
        if !gyro_settings.enabled && !gyro_mouse_active {
            return;
        }
        
        // Apply sensitivity factor
        let sensitivity_factor = self.get_sensitivity_factor();
        
        // Map gyro to mouse movement, this is button face up behavior
        let mut dx = y * gyro_settings.sensitivity_x * sensitivity_factor;
        let mut dy = -x * gyro_settings.sensitivity_y * sensitivity_factor; 
        
        if gyro_settings.invert_x {
            dx = -dx;
        }
        if gyro_settings.invert_y {
            dy = -dy;
        }
        
        let dx_i = dx as i32;
        let dy_i = dy as i32;
        
        if dx_i != 0 || dy_i != 0 {
            if let Err(e) = self.mouse.move_relative(dx_i, dy_i) {
                warn!("Failed to move mouse (gyro): {}", e);
            }
        }
    }
    
    /// Handle full state update
    fn on_state_update(&mut self, state: &JoyConState) {
        // Update held button states
        self.sync_button_states(state);
        
        // Store previous state
        self.previous_state = state.clone();
    }
    
    /// Execute an action (press or release), for keyhold, this will call held_state methods
    fn execute_action(&mut self, action: &Action, pressed: bool, _side: ControllerSide) {
        match action {
            Action::None { .. } => {
                // Explicitly do nothing
            }

            // Key hold actions, call held_state methods
            Action::KeyHold { key } => {
                // Skip if key is None or empty string
                let Some(key_name) = key else {
                    return;
                };
                
                // Also skip if key is an empty string
                if key_name.is_empty() {
                    return;
                }
                
                // Check if this is a multi-key combo (e.g., "shift+w")
                let keys: Vec<&str> = key_name.split('+').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                if pressed {
                    for k in &keys { self.held_state.press_key(k, KeySource::Button, &self.keyboard); }
                } else {
                    for k in keys.iter().rev() { self.held_state.release_key(k, KeySource::Button, &self.keyboard); }
                }
            }
            
            Action::MouseMove { dx, dy } => {
                if pressed {
                    if let Err(e) = self.mouse.move_relative(*dx, *dy) {
                        warn!("Failed to move mouse: {}", e);
                    }
                }
            }
            
            Action::MouseClick { button } => {
                let btn = match button {
                    crate::mapping::config::MouseButton::Left => MouseButton::Left,
                    crate::mapping::config::MouseButton::Right => MouseButton::Right,
                    crate::mapping::config::MouseButton::Middle => MouseButton::Middle,
                };
                
                if pressed {
                    if let Err(e) = self.mouse.button_down(btn) {
                        warn!("Failed to press mouse button: {}", e);
                    }
                } else {
                    if let Err(e) = self.mouse.button_up(btn) {
                        warn!("Failed to release mouse button: {}", e);
                    }
                }
            }
            
            Action::CycleProfiles => {
                if pressed {
                    self.cycle_profiles();
                }
            }
            
            Action::CycleSensitivity => {
                if pressed {
                    self.cycle_sensitivity();
                }
            }
            
            Action::ToggleGyroMouseL => {
                if pressed {
                    self.toggle_gyro_mouse(ControllerSide::Left);
                }
            }
            
            Action::ToggleGyroMouseR => {
                if pressed {
                    self.toggle_gyro_mouse(ControllerSide::Right);
                }
            }
        }
    }
    
    /// Cycle to the next profile
    fn cycle_profiles(&mut self) {
        if self.config.profiles.is_empty() {
            return;
        }
        
        let old_index = self.current_profile_index;
        let old_name = self.config.profiles[old_index].name.clone();
        
        // Cycle to next profile
        self.current_profile_index = (self.current_profile_index + 1) % self.config.profiles.len();
        
        let new_name = self.config.profiles[self.current_profile_index].name.clone();
        
        info!("🔄 Cycled profile: '{}' -> '{}'", old_name, new_name);
        
        // Release all held keys when switching profiles
        self.release_all_held_keys();
    }
    
    /// Cycle through sensitivity factors
    fn cycle_sensitivity(&mut self) {
        if self.config.settings.sensitivity_factor.is_empty() {
            return;
        }
        
        let old_index = self.current_sensitivity_index;
        self.current_sensitivity_index = 
            (self.current_sensitivity_index + 1) % self.config.settings.sensitivity_factor.len();
        
        let old_factor = self.config.settings.sensitivity_factor[old_index];
        let new_factor = self.config.settings.sensitivity_factor[self.current_sensitivity_index];
        
        info!("🎯 Sensitivity: {:.1}x -> {:.1}x", old_factor, new_factor);
    }
    
    /// Toggle gyro mouse for a controller side
    fn toggle_gyro_mouse(&mut self, side: ControllerSide) {
        let enabled = match side {
            ControllerSide::Left => {
                self.gyro_mouse_state.left_enabled = !self.gyro_mouse_state.left_enabled;
                self.gyro_mouse_state.left_enabled
            }
            ControllerSide::Right => {
                self.gyro_mouse_state.right_enabled = !self.gyro_mouse_state.right_enabled;
                self.gyro_mouse_state.right_enabled
            }
        };
        
        info!("🎮 Gyro mouse {:?}: {}", side, if enabled { "ENABLED" } else { "DISABLED" });
    }
    
    /// Handle directional keys for stick movement
    fn handle_directional_keys(
        &mut self,
        x: f32,
        y: f32,
        directions: &crate::mapping::config::DirectionalKeys,
    ) {
        // Determine which keys should be pressed based on stick position
        let threshold = 0.5;
        
        // Note: Y-axis is inverted on controllers - negative Y is UP, positive Y is DOWN
        let should_press_up = y < -threshold;
        let should_press_down = y > threshold;
        let should_press_left = x < -threshold;
        let should_press_right = x > threshold;
        
        // Press/release keys accordingly
        self.set_stick_key_state(&directions.up, should_press_up);
        self.set_stick_key_state(&directions.down, should_press_down);
        self.set_stick_key_state(&directions.left, should_press_left);
        self.set_stick_key_state(&directions.right, should_press_right);
    }
    
    /// Set key state for stick source (press or release). Ensures we don't release a key still held by a button.
    fn set_stick_key_state(&mut self, key: &str, pressed: bool) {
        if key.is_empty() { return; }
        let keys: Vec<&str> = key.split('+').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        if pressed {
            for k in &keys { self.held_state.press_key(k, KeySource::Stick, &self.keyboard); }
        } else {
            for k in keys.iter().rev() { self.held_state.release_key(k, KeySource::Stick, &self.keyboard); }
        }
    }
    
    /// Release all directional keys for a stick
    fn release_directional_keys(&mut self, stick: StickType) {
        let profile = match self.current_profile() {
            Some(p) => p,
            None => return,
        };
        
        let mapping = match stick {
            StickType::Left => profile.sticks.left.as_ref(),
            StickType::Right => profile.sticks.right.as_ref(),
        };
        
        if let Some(mapping) = mapping {
            if let Some(directions) = &mapping.directions {
                let keys = vec![
                    directions.up.clone(),
                    directions.down.clone(),
                    directions.left.clone(),
                    directions.right.clone(),
                ];
                for key in keys {
                    self.set_stick_key_state(&key, false);
                }
            }
        }
    }
    
    /// Sync button states with current Joy-Con state
    fn sync_button_states(&mut self, _buttons: &JoyConState) {
        // This is called on every state update to ensure consistency
        // (In case we missed a button event)
    }
    
    /// Release all currently held keys (e.g., on disconnect or profile switch)
    fn release_all_held_keys(&mut self) { self.held_state.clear_all(&self.keyboard); }
}
