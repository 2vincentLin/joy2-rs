//! Example 11: Gyroscope to Mock Mouse
//!
//! This example demonstrates using the Right Joy-Con 2 gyroscope to control
//! a mock mouse backend. Press B button to toggle mouse output on/off.

use btleplug::api::Peripheral as _;
use futures::stream::StreamExt;
use joy2_rs::backend::{MockMouseBackend, MouseBackend};
use joy2_rs::joycon2::connection::{init_controller, Side};
use joy2_rs::joycon2::controller::Joy2R;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Starting Joy-Con 2 Right gyroscope to mock mouse example...");

    println!("Joy-Con 2 Right - Gyroscope to Mock Mouse");
    println!("==========================================\n");
    println!("This example will use the gyroscope to control mock mouse movement.");
    println!("Press B button to toggle mouse output ON/OFF.");
    println!("Press Ctrl+C to exit.\n");

    println!("Press the sync button on your Joy-Con Right now...\n");

    // Initialize the controller
    let connection = init_controller(Side::Right).await?;

    println!("\n✓ Controller initialized! Starting gyro mouse control...\n");
    println!("🎮 Controls:");
    println!("  • Tilt/rotate the controller to move the mouse");
    println!("  • Press B to toggle mouse output ON/OFF");
    println!("  • Press Ctrl+C to exit");
    println!("\n=================================================\n");

    // Create mock mouse backend
    let mouse = MockMouseBackend::new();

    // Create controller state tracker
    let mut controller = Joy2R::new();

    // Get peripheral and subscribe to notifications
    let peripheral = connection.peripheral();
    let mut notification_stream = peripheral.notifications().await?;

    // State tracking
    let mut mouse_enabled = true;
    let mut prev_b_button = false;

    // Gyroscope sensitivity (how much to scale gyro values for mouse movement)
    let gyro_sensitivity = 2.0;

    println!("🟢 Mouse output: ENABLED (press B to toggle)\n");

    // Process notifications
    while let Some(notification) = notification_stream.next().await {
        // Update controller state
        controller.update(&notification.value);

        let buttons = &controller.buttons;
        let gyro = &controller.gyroscope;

        // Check if B button was pressed (rising edge)
        if buttons.b && !prev_b_button {
            mouse_enabled = !mouse_enabled;
            if mouse_enabled {
                println!("\n🟢 Mouse output: ENABLED\n");
            } else {
                println!("\n🔴 Mouse output: DISABLED\n");
            }
        }
        prev_b_button = buttons.b;

        // Process gyroscope data if mouse is enabled
        if mouse_enabled {
            // Map gyroscope to mouse movement:
            // Gyro Z (yaw - rotating left/right) -> mouse horizontal (dx)
            // Gyro Y (pitch - tilting up/down) -> mouse vertical (dy)
            // Note: Gyro X (roll) is not typically used for mouse control
            
            // let dx = (gyro.z * gyro_sensitivity) as i32;
            let dx = (gyro.y * gyro_sensitivity) as i32;
            let dy = (-gyro.x * gyro_sensitivity) as i32; // Invert X for natural movement

            // Only send mouse movement if there's significant motion
            let threshold = 1; // minimum movement threshold
            if dx.abs() >= threshold || dy.abs() >= threshold {
                if let Err(e) = mouse.move_relative(dx, dy) {
                    log::warn!("Failed to move mouse: {}", e);
                }
            }
        }
    }

    Ok(())
}
