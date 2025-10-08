//! Test config validation and logging
//!
//! This example demonstrates the enhanced config validation including:
//! - Key name validation against keyboard backend
//! - Profile switching button consistency checks
//! - Multi-key combo validation
//! - Detailed logging during config load

use joy2_rs::mapping::config::Config;

fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();
    
    println!("Testing config validation with ETS2.toml...\n");
    
    // Load ETS2 config
    match Config::load("configs/ETS2.toml") {
        Ok(config) => {
            println!("\n✅ Config loaded successfully!");
            println!("   Profiles:");
            for profile in &config.profiles {
                println!("     - {}: {}", profile.name, profile.description);
                println!("       Buttons mapped: {}", profile.buttons.len());
                println!("       Gyro overrides (L): {}", profile.gyro_mouse_overrides_left.len());
                println!("       Gyro overrides (R): {}", profile.gyro_mouse_overrides_right.len());
            }
        }
        Err(e) => {
            eprintln!("\n❌ Config validation failed: {}", e);
            std::process::exit(1);
        }
    }
    
    println!("\n---\n");
    
    // Try loading default config (should fail if it doesn't exist)
    println!("Testing default.toml (if exists)...\n");
    match Config::load_default() {
        Ok(config) => {
            println!("✅ Default config loaded with {} profiles", config.profiles.len());
        }
        Err(e) => {
            println!("⚠️  Default config not found (this is OK): {}", e);
        }
    }
}
