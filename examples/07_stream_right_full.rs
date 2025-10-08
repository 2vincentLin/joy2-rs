use btleplug::api::Peripheral as _;
use futures::stream::StreamExt;
use joy2_rs::joycon2::connection::{init_controller, Side};
use joy2_rs::joycon2::controller::Joy2R;
use std::error::Error;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    log::info!("Starting Joy-Con 2 Right full data streaming example...");

    println!("Joy-Con 2 Right - Full Data Streaming");
    println!("======================================\n");
    println!("This example will stream buttons, stick, and motion data from Joy-Con Right.");
    println!("Press Ctrl+C to exit.\n");
    
    println!("Press the sync button on your Joy-Con Right now...\n");
    
    // Initialize the controller
    let connection = init_controller(Side::Right).await?;
    
    println!("\n✓ Controller initialized! Starting full data stream...\n");
    println!("Try pressing buttons, moving the stick, and moving the controller!");
    println!("\n=================================================\n");
    
    // Create controller state tracker
    let mut controller = Joy2R::new();
    
    // Get peripheral and subscribe to notifications
    let peripheral = connection.peripheral();
    let mut notification_stream = peripheral.notifications().await?;
    
    println!("Listening for input... (Press Ctrl+C to exit)\n");
    
    println!("Legend:");
    println!("  🎮 Buttons: A, B, X, Y, R, ZR, +, Home, Chat, SL, SR, R3");
    println!("  🕹️  Right Stick: X/Y axis from -1.0 to +1.0");
    println!("  🔄 Gyroscope: Rotation in degrees/second (X/Y/Z)");
    println!("  📐 Accelerometer: Linear acceleration in G-force (X/Y/Z)");
    println!("\n=================================================\n");
    
    let mut last_update = Instant::now();
    let update_interval = Duration::from_millis(100); // Update every 100ms
    
    let mut last_buttons_display = String::new();
    let mut last_stick_display = String::new();
    
    // Process notifications
    while let Some(notification) = notification_stream.next().await {
        // Update controller state
        controller.update(&notification.value);
        
        let now = Instant::now();
        
        // Only display every 100ms to avoid spam
        if now.duration_since(last_update) >= update_interval {
            let buttons = &controller.buttons;
            let stick = &controller.analog_stick;
            let gyro = &controller.gyroscope;
            let accel = &controller.accelerometer;
            
            // Check for button changes
            let button_parts: Vec<&str> = vec![
                if buttons.a { "A" } else { "" },
                if buttons.b { "B" } else { "" },
                if buttons.x { "X" } else { "" },
                if buttons.y { "Y" } else { "" },
                if buttons.r { "R" } else { "" },
                if buttons.zr { "ZR" } else { "" },
                if buttons.plus { "+" } else { "" },
                if buttons.home { "🏠" } else { "" },
                if buttons.chat { "💬" } else { "" },
                if buttons.r3 { "R3" } else { "" },
                if buttons.srr { "SR" } else { "" },
                if buttons.slr { "SL" } else { "" },
            ];
            
            let active_buttons: Vec<&str> = button_parts.into_iter().filter(|s| !s.is_empty()).collect();
            let buttons_display = if active_buttons.is_empty() {
                "(none)".to_string()
            } else {
                active_buttons.join(" + ")
            };
            
            // Check for stick changes (with deadzone)
            let deadzone = 0.1;
            let stick_display = if stick.x.abs() < deadzone && stick.y.abs() < deadzone {
                "centered".to_string()
            } else {
                let direction = if stick.y > 0.5 {
                    " [↑]"
                } else if stick.y < -0.5 {
                    " [↓]"
                } else if stick.x > 0.5 {
                    " [→]"
                } else if stick.x < -0.5 {
                    " [←]"
                } else {
                    ""
                };
                format!("X={:+.2}, Y={:+.2}{}", stick.x, stick.y, direction)
            };
            
            // Display buttons if changed
            if buttons_display != last_buttons_display {
                println!("🎮 Buttons: {}", buttons_display);
                last_buttons_display = buttons_display;
            }
            
            // Display stick if changed
            if stick_display != last_stick_display {
                println!("🕹️  Right Stick: {}", stick_display);
                last_stick_display = stick_display;
            }
            
            // Display motion data every update (always interesting)
            let motion_threshold = 5.0; // degrees/second for gyro
            let accel_threshold = 0.15; // G-force
            
            let has_gyro_activity = gyro.x.abs() > motion_threshold 
                || gyro.y.abs() > motion_threshold 
                || gyro.z.abs() > motion_threshold;
                
            let has_accel_activity = accel.x.abs() > accel_threshold 
                || (accel.y + 1.0).abs() > accel_threshold // Y should be ~-1.0 at rest
                || accel.z.abs() > accel_threshold;
            
            if has_gyro_activity || has_accel_activity {
                println!("\n┌─────────────────────────────────────────────────────────┐");
                println!("│ 🔄 GYROSCOPE (degrees/second)");
                
                let gyro_x_bar = create_bar(gyro.x, 100.0);
                let gyro_y_bar = create_bar(gyro.y, 100.0);
                let gyro_z_bar = create_bar(gyro.z, 100.0);
                
                println!("│   X: {:>8.2}°/s {}", gyro.x, gyro_x_bar);
                println!("│   Y: {:>8.2}°/s {}", gyro.y, gyro_y_bar);
                println!("│   Z: {:>8.2}°/s {}", gyro.z, gyro_z_bar);
                
                println!("├─────────────────────────────────────────────────────────┤");
                println!("│ 📐 ACCELEROMETER (G-force)");
                
                let x_bar = create_bar(accel.x, 2.0);
                let y_bar = create_bar(accel.y + 1.0, 2.0); // Offset by +1.0 since rest is -1.0
                let z_bar = create_bar(accel.z, 2.0);
                
                println!("│   X: {:>7.2}G {}", accel.x, x_bar);
                println!("│   Y: {:>7.2}G {}", accel.y, y_bar);
                println!("│   Z: {:>7.2}G {}", accel.z, z_bar);
                
                println!("└─────────────────────────────────────────────────────────┘");
                
                // Activity indicators
                let mut activities = Vec::new();
                
                if gyro.x > motion_threshold {
                    activities.push("🔄 Gyro X+");
                } else if gyro.x < -motion_threshold {
                    activities.push("🔄 Gyro X-");
                }
                
                if gyro.y > motion_threshold {
                    activities.push("🔄 Gyro Y+");
                } else if gyro.y < -motion_threshold {
                    activities.push("🔄 Gyro Y-");
                }
                
                if gyro.z > motion_threshold {
                    activities.push("🔄 Gyro Z+");
                } else if gyro.z < -motion_threshold {
                    activities.push("🔄 Gyro Z-");
                }
                
                if accel.x > accel_threshold {
                    activities.push("📐 Accel X+");
                } else if accel.x < -accel_threshold {
                    activities.push("📐 Accel X-");
                }
                
                if accel.y > -1.0 + accel_threshold {
                    activities.push("📐 Accel Y+");
                } else if accel.y < -1.0 - accel_threshold {
                    activities.push("📐 Accel Y-");
                }
                
                if accel.z > accel_threshold {
                    activities.push("📐 Accel Z+");
                } else if accel.z < -accel_threshold {
                    activities.push("📐 Accel Z-");
                }
                
                if !activities.is_empty() {
                    println!("🟢 Active: {}\n", activities.join(" | "));
                } else {
                    println!();
                }
            }
            
            last_update = now;
        }
    }
    
    Ok(())
}

/// Create a visual bar indicator for a value
fn create_bar(value: f32, max: f32) -> String {
    let normalized = (value / max).clamp(-1.0, 1.0);
    let bar_width = 30;
    let center = bar_width / 2;
    
    if normalized.abs() < 0.05 {
        return format!("{:width$}│{:width$}", "", "", width = center);
    }
    
    let pos = ((normalized + 1.0) / 2.0 * bar_width as f32) as usize;
    let pos = pos.min(bar_width - 1);
    
    let mut bar = vec![' '; bar_width];
    bar[center] = '│';
    
    if pos > center {
        for i in (center + 1)..=pos {
            bar[i] = '►';
        }
    } else if pos < center {
        for i in pos..center {
            bar[i] = '◄';
        }
    }
    
    bar.into_iter().collect()
}
