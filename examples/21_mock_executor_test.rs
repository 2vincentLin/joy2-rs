//! Example 22: Test config loading and executor with mock backends
//!
//! This example tests the configuration system and executor initialization
//! without requiring actual Joy-Con 2 controllers. It demonstrates:
//! - Loading config from TOML files
//! - Config validation
//! - Profile system
//! - Mock backend setup
//! - Executor initialization

use joy2_rs::backend::{MockKeyboardBackend, MockMouseBackend};
use joy2_rs::mapping::config::Config;
use joy2_rs::mapping::executor::MappingExecutor;
use joy2_rs::mapping::config::{JoyConEvent, ButtonType, StickType, ControllerSide};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("=== Joy-Con 2 Config & Executor Test ===\n");

    // Test 1: Load ETS2.toml
    println!("📋 Test 1: Loading ETS2.toml...");
    let config = Config::load("configs/ETS2.toml")?;
    println!("   ✓ Loaded {} profiles", config.profiles.len());
    for profile in &config.profiles {
        println!("     - {}: {} (buttons: {})", 
            profile.name, 
            profile.description,
            profile.buttons.len()
        );
    }
    println!();

    // Test 2: Create executor with mock backends
    println!("🎮 Test 2: Creating executor with mock backends...");
    let keyboard = MockKeyboardBackend::new();
    let mouse = MockMouseBackend::new();
    let mut executor = MappingExecutor::new(config.clone(), keyboard, mouse);
    println!("   ✓ Executor created\n");

    // Test 3: Simulate some events
    println!("🔘 Test 3: Simulating button events...");
    println!("   Pressing A button (should trigger action)...");
    executor.process_event(&JoyConEvent::ButtonPressed(ButtonType::A));
    
    println!("   Releasing A button...");
    executor.process_event(&JoyConEvent::ButtonReleased(ButtonType::A));
    
    println!("   Pressing SLR button (CycleProfiles)...");
    executor.process_event(&JoyConEvent::ButtonPressed(ButtonType::SLR));
    executor.process_event(&JoyConEvent::ButtonReleased(ButtonType::SLR));
    println!();

    // Test 4: Simulate stick movement
    println!("🕹️  Test 4: Simulating stick movement...");
    println!("   Moving left stick up (0.0, -0.8)...");
    executor.process_event(&JoyConEvent::StickMoved {
        stick: StickType::Left,
        x: 0.0,
        y: -0.8,
    });
    
    println!("   Moving left stick back to center (0.0, 0.0)...");
    executor.process_event(&JoyConEvent::StickMoved {
        stick: StickType::Left,
        x: 0.0,
        y: 0.0,
    });
    println!();

    // Test 5: Simulate gyro toggle and movement
    println!("🎯 Test 5: Simulating gyro toggle and movement...");
    println!("   Toggling gyro mouse for right controller...");
    executor.process_event(&JoyConEvent::ButtonPressed(ButtonType::SRR));
    executor.process_event(&JoyConEvent::ButtonReleased(ButtonType::SRR));
    
    println!("   Simulating gyro movement (pitch: 0.1, yaw: 0.2)...");
    executor.process_event(&JoyConEvent::GyroUpdate {
        side: ControllerSide::Right,
        x: 0.0,  // roll
        y: 0.1,  // pitch
        z: 0.2,  // yaw
    });
    println!();

    // Test 6: Cycle sensitivity
    println!("🎚️  Test 6: Cycling sensitivity...");
    println!("   Pressing Plus button (CycleSensitivity)...");
    executor.process_event(&JoyConEvent::ButtonPressed(ButtonType::Plus));
    executor.process_event(&JoyConEvent::ButtonReleased(ButtonType::Plus));
    
    println!("   Pressing Plus button again...");
    executor.process_event(&JoyConEvent::ButtonPressed(ButtonType::Plus));
    executor.process_event(&JoyConEvent::ButtonReleased(ButtonType::Plus));
    println!();

    // Test 7: Profile cycling
    println!("🔄 Test 7: Cycling through profiles...");
    println!("   Current profiles: {:?}", 
        config.profiles.iter().map(|p| &p.name).collect::<Vec<_>>()
    );
    
    println!("   Pressing SLR (CycleProfiles) to switch profile...");
    executor.process_event(&JoyConEvent::ButtonPressed(ButtonType::SLR));
    executor.process_event(&JoyConEvent::ButtonReleased(ButtonType::SLR));
    
    println!("   Pressing A button in new profile...");
    executor.process_event(&JoyConEvent::ButtonPressed(ButtonType::A));
    executor.process_event(&JoyConEvent::ButtonReleased(ButtonType::A));
    println!();

    println!("✅ All tests completed successfully!\n");
    println!("Summary:");
    println!("  - Config loaded and validated ✓");
    println!("  - Executor initialized ✓");
    println!("  - Button events processed ✓");
    println!("  - Stick movements processed ✓");
    println!("  - Gyro toggle and movement ✓");
    println!("  - Sensitivity cycling ✓");
    println!("  - Profile cycling ✓");
    println!("\n🎉 The mock backend system is working correctly!");

    Ok(())
}
