//! Example 08: Test manager with mock backends
//!
//! This example demonstrates the JoyConManager with mock backends
//! that print events instead of sending actual keyboard/mouse input.
//! Great for testing the manager without affecting your system.

use joy2_rs::backend::{MockKeyboardBackend, MockMouseBackend};
use joy2_rs::mapping::config::Config;
use joy2_rs::JoyConManager;
use std::error::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("=== Joy-Con 2 Manager Test with Mock Backends ===");
    println!();
    println!("This example will:");
    println!("1. Scan for Joy-Con 2 controllers (Left and Right)");
    println!("2. Connect to them");
    println!("3. Print all button presses, stick movements, and gyro data");
    println!("4. Show what keyboard/mouse events WOULD be sent (mock output)");
    println!();
    println!("Press Ctrl+C to stop");
    println!();

    // Try to load ETS2.toml first (more complete example), then default.toml
    let config = if let Ok(cfg) = Config::load("configs/ETS2.toml") {
        println!("✓ Loaded configuration from configs/ETS2.toml");
        cfg
    } else if let Ok(cfg) = Config::load_default() {
        println!("✓ Loaded configuration from configs/default.toml");
        cfg
    } else {
        eprintln!("✗ Failed to load config files");
        eprintln!("  Creating fallback configuration...");
        
        // Fallback: Create a simple configuration with profile-based mappings
        use joy2_rs::mapping::config::{
            Action, ButtonType, Profile, Settings, StickMappings, GyroSettings,
            StickMapping, StickMode, DirectionalKeys,
        };
        use std::collections::HashMap;
        
        let mut buttons = HashMap::new();
        
        // Map some buttons to keyboard keys
        buttons.insert(ButtonType::A, vec![Action::KeyHold {
            key: Some("space".to_string()),
        }]);
        
        buttons.insert(ButtonType::B, vec![Action::KeyHold {
            key: Some("w".to_string()),
        }]);
        
        buttons.insert(ButtonType::X, vec![Action::KeyHold {
            key: Some("a".to_string()),
        }]);
        
        buttons.insert(ButtonType::Y, vec![Action::KeyHold {
            key: Some("s".to_string()),
        }]);
        
        // Add profile cycling button
        buttons.insert(ButtonType::SLR, vec![Action::CycleProfiles]);
        
        // Add gyro mouse toggle
        buttons.insert(ButtonType::SRR, vec![Action::ToggleGyroMouseR]);
        
        // Create a base profile
        let base_profile = Profile {
            name: "base".to_string(),
            description: "Fallback test profile".to_string(),
            buttons: buttons.clone(),
            sticks: StickMappings {
                left: Some(StickMapping {
                    mode: StickMode::Directional,
                    sensitivity: 1.0,
                    directions: Some(DirectionalKeys {
                        up: "w".to_string(),
                        down: "s".to_string(),
                        left: "a".to_string(),
                        right: "d".to_string(),
                    }),
                }),
                right: Some(StickMapping {
                    mode: StickMode::Mouse,
                    sensitivity: 1.0,
                    directions: None,
                }),
            },
            gyro: GyroSettings::default(),
            gyro_mouse_overrides_left: HashMap::new(),
            gyro_mouse_overrides_right: HashMap::new(),
        };
        
        Config {
            settings: Settings::default(),
            profiles: vec![base_profile],
        }
    };

    // Create mock backends
    let keyboard = MockKeyboardBackend::new();
    let mouse = MockMouseBackend::new();

    // Create the manager
    let mut manager = JoyConManager::new(config, keyboard, mouse);

    // Start the manager (spawns threads for executor and controllers)
    println!("Starting manager...");
    manager.start()?;

    println!("Manager started! Waiting for controller events...");
    println!();

    // Keep the main thread alive
    // In a real application, you'd handle Ctrl+C gracefully
    loop {
        thread::sleep(Duration::from_secs(1));

        if !manager.is_running() {
            println!("Manager stopped");
            break;
        }
    }

    Ok(())
}
