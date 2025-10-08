use btleplug::api::Peripheral as _;
use futures::stream::StreamExt;
use joy2_rs::joycon2::connection::{init_controller, Side};
use joy2_rs::joycon2::controller::Joy2L;
use std::error::Error;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    println!("Joy-Con 2 Motion Data Streaming Example");
    println!("========================================\n");
    println!("This example will stream gyroscope and accelerometer data.");
    println!("Press Ctrl+C to exit.\n");
    
    println!("Press the sync button on your Joy-Con Left now...\n");
    
    // Initialize the controller
    let connection = init_controller(Side::Left).await?;
    
    println!("\n✓ Controller initialized! Starting motion data stream...\n");
    println!("Try moving, rotating, and tilting the controller!\n");
    println!("=================================================\n");
    
    // Create controller state tracker
    let mut controller = Joy2L::new();
    
    // Get peripheral and subscribe to notifications
    let peripheral = connection.peripheral();
    let mut notification_stream = peripheral.notifications().await?;
    
    // Display configuration
    const UPDATE_INTERVAL: Duration = Duration::from_secs(1); // Update display every 1 second
    const MOTION_THRESHOLD_GYRO: f32 = 5.0;  // Degrees per second
    const MOTION_THRESHOLD_ACCEL: f32 = 0.15; // G-force
    
    let mut last_display = Instant::now();
    let mut motion_detected = false;
    let mut frame_count = 0u32;
    
    println!("Listening for motion... (Press Ctrl+C to exit)\n");
    println!("Legend:");
    println!("  🔄 Gyroscope: Rotation in degrees/second");
    println!("    • Pitch (X): Tilt forward/backward");
    println!("    • Roll  (Y): Tilt left/right");
    println!("    • Yaw   (Z): Twist clockwise/counterclockwise");
    println!("  📐 Accelerometer: Linear acceleration in G-force (1G = 9.8 m/s²)");
    println!("    • X: Left/Right");
    println!("    • Y: Up/Down");
    println!("    • Z: Forward/Backward");
    println!("\n=================================================\n");
    
    // Process notifications
    while let Some(notification) = notification_stream.next().await {
        frame_count += 1;
        
        // Update controller state
        controller.update(&notification.value);
        
        // Display motion data at regular intervals
        if last_display.elapsed() >= UPDATE_INTERVAL {
            println!("═══════════════════════════════════════════════════════════");
            println!("Frame: {} | Timestamp: {} | Packet len: {}", 
                frame_count, controller.timestamp, notification.value.len());
            println!("═══════════════════════════════════════════════════════════\n");
            
            // Print raw packet bytes (first 64 bytes or all if shorter)
            println!("📦 RAW PACKET DATA (hex):");
            let display_len = notification.value.len().min(64);
            for chunk in notification.value[..display_len].chunks(16) {
                print!("  ");
                for byte in chunk {
                    print!("{:02X} ", byte);
                }
                println!();
            }
            if notification.value.len() > 64 {
                println!("  ... ({} more bytes)", notification.value.len() - 64);
            }
            println!();
            
            // Print RAW data first
            let gyro = &controller.gyroscope;
            let accel = &controller.accelerometer;
            
            println!("📊 RAW SENSOR DATA:");
            println!("  Gyroscope:");
            println!("    X (Pitch): {:.6}", gyro.x);
            println!("    Y (Roll):  {:.6}", gyro.y);
            println!("    Z (Yaw):   {:.6}", gyro.z);
            println!("  Accelerometer:");
            println!("    X: {:.6}", accel.x);
            println!("    Y: {:.6}", accel.y);
            println!("    Z: {:.6}", accel.z);
            println!();
            
            // Print formatted version
            print_motion_data(&controller);
            
            // Check motion thresholds
            let has_rotation = gyro.x.abs() > MOTION_THRESHOLD_GYRO 
                || gyro.y.abs() > MOTION_THRESHOLD_GYRO 
                || gyro.z.abs() > MOTION_THRESHOLD_GYRO;
            
            let has_movement = (accel.x.abs() - 1.0).abs() > MOTION_THRESHOLD_ACCEL 
                || accel.y.abs() > MOTION_THRESHOLD_ACCEL 
                || accel.z.abs() > MOTION_THRESHOLD_ACCEL;
            
            let current_motion = has_rotation || has_movement;
            
            if current_motion {
                print_motion_activity(gyro.x, gyro.y, gyro.z, accel.x, accel.y, accel.z);
            } else if motion_detected {
                println!("⚪ Motion stopped - controller at rest\n");
            }
            
            motion_detected = current_motion;
            last_display = Instant::now();
        }
    }
    
    Ok(())
}

/// Print motion data in a formatted way
fn print_motion_data(controller: &Joy2L) {
    let gyro = &controller.gyroscope;
    let accel = &controller.accelerometer;
    
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│ 🔄 GYROSCOPE (degrees/second)                           │");
    println!("│   Pitch (X): {:+7.2}°/s  {}                     │", 
        gyro.x, format_rotation_bar(gyro.x, 200.0));
    println!("│   Roll  (Y): {:+7.2}°/s  {}                     │", 
        gyro.y, format_rotation_bar(gyro.y, 200.0));
    println!("│   Yaw   (Z): {:+7.2}°/s  {}                     │", 
        gyro.z, format_rotation_bar(gyro.z, 200.0));
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│ 📐 ACCELEROMETER (G-force)                              │");
    println!("│   X: {:+5.2}G  {}                               │", 
        accel.x, format_accel_bar(accel.x, 2.0));
    println!("│   Y: {:+5.2}G  {}                               │", 
        accel.y, format_accel_bar(accel.y, 2.0));
    println!("│   Z: {:+5.2}G  {}                               │", 
        accel.z, format_accel_bar(accel.z, 2.0));
    println!("└─────────────────────────────────────────────────────────┘");
}

/// Print motion activity indicators
fn print_motion_activity(pitch: f32, roll: f32, yaw: f32, ax: f32, ay: f32, az: f32) {
    let mut activities = Vec::new();
    
    // Gyroscope activities
    if pitch.abs() > 50.0 {
        if pitch > 0.0 {
            activities.push("🔄 Tilting FORWARD");
        } else {
            activities.push("🔄 Tilting BACKWARD");
        }
    }
    
    if roll.abs() > 50.0 {
        if roll > 0.0 {
            activities.push("🔄 Tilting LEFT");
        } else {
            activities.push("🔄 Tilting RIGHT");
        }
    }
    
    if yaw.abs() > 50.0 {
        if yaw > 0.0 {
            activities.push("🔄 Twisting CLOCKWISE");
        } else {
            activities.push("🔄 Twisting COUNTER-CLOCKWISE");
        }
    }
    
    // Accelerometer activities (subtract gravity on Z-axis)
    if ax.abs() > 0.3 {
        if ax > 0.0 {
            activities.push("📐 Moving RIGHT");
        } else {
            activities.push("📐 Moving LEFT");
        }
    }
    
    if ay.abs() > 0.3 {
        if ay > 0.0 {
            activities.push("📐 Moving UP");
        } else {
            activities.push("📐 Moving DOWN");
        }
    }
    
    if (az.abs() - 1.0).abs() > 0.3 {
        if az > 1.0 {
            activities.push("📐 Accelerating FORWARD");
        } else if az < 1.0 {
            activities.push("📐 Accelerating BACKWARD");
        }
    }
    
    if !activities.is_empty() {
        println!("🟢 Active: {}", activities.join(" | "));
    } else {
        println!("🟡 Minor motion detected");
    }
    println!();
}

/// Format a rotation value as a bar chart
fn format_rotation_bar(value: f32, max: f32) -> String {
    let width = 20;
    let normalized = (value / max).clamp(-1.0, 1.0);
    let pos = ((normalized + 1.0) / 2.0 * width as f32) as usize;
    
    let mut bar = vec![' '; width];
    bar[width / 2] = '│';
    
    if pos < width / 2 {
        for i in pos..width/2 {
            bar[i] = '◄';
        }
    } else if pos > width / 2 {
        for i in width/2+1..=pos.min(width-1) {
            bar[i] = '►';
        }
    }
    
    bar.iter().collect()
}

/// Format an acceleration value as a bar chart
fn format_accel_bar(value: f32, max: f32) -> String {
    let width = 20;
    let normalized = (value / max).clamp(-1.0, 1.0);
    let pos = ((normalized + 1.0) / 2.0 * width as f32) as usize;
    
    let mut bar = vec![' '; width];
    bar[width / 2] = '│';
    
    if pos < width / 2 {
        for i in pos..width/2 {
            bar[i] = '◄';
        }
    } else if pos > width / 2 {
        for i in width/2+1..=pos.min(width-1) {
            bar[i] = '►';
        }
    }
    
    bar.iter().collect()
}
