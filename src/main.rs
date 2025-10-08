//! Joy-Con 2 Manager - Main Application
//!
//! This is the main entry point for the Joy-Con 2 controller manager.
//! It uses REAL keyboard/mouse backends that send input to your system.
//!
//! ⚠️  WARNING: This will send REAL keyboard and mouse input to your system!
//! ⚠️  Make sure you have your config set up correctly before running.

use joy2_rs::backend::{KeyboardSendInputBackend, MouseSendInputBackend};
use joy2_rs::mapping::config::Config;
use joy2_rs::JoyConManager;
use std::error::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!("=== Joy-Con 2 Manager ===");
    println!();
    println!("⚠️  WARNING: This uses REAL keyboard/mouse input!");
    println!("⚠️  Your Joy-Con inputs will control your system!");
    println!();
    println!("This application will:");
    println!("1. Scan for Joy-Con 2 controllers (Left and Right)");
    println!("2. Connect to both controllers");
    println!("3. Send REAL keyboard and mouse input based on your config");
    println!("4. Cache controller MAC addresses for faster reconnection");
    println!();
    println!("please visit https://github.com/2vincentLin/joy2-rs for more information");
    println!();
    println!("Press Ctrl+C to stop");
    println!();

    // Load default configuration
    let config = Config::load_default()?;
    println!("✓ Loaded configuration from configs/default.toml");

    // Create real backends (unit structs - no new() needed)
    let keyboard = KeyboardSendInputBackend;
    let mouse = MouseSendInputBackend;

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
