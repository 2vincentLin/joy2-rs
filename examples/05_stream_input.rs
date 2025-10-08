use btleplug::api::Peripheral as _;
use futures::stream::StreamExt;
use joy2_rs::joycon2::connection::{init_controller, Side};
use joy2_rs::joycon2::controller::Joy2L;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    log::info!("Starting Joy-Con 2 input streaming example...");

    println!("Joy-Con 2 Input Streaming Example");
    println!("==================================\n");
    println!("This example will stream button and stick data from the Joy-Con.");
    println!("Press Ctrl+C to exit.\n");
    
    println!("Press the sync button on your Joy-Con Left now...\n");
    
    // Initialize the controller
    let connection = init_controller(Side::Left).await?;
    
    println!("\n✓ Controller initialized! Starting input stream...\n");
    println!("Try pressing buttons and moving the stick!\n");
    println!("=================================================\n");
    
    // Create controller state tracker
    let mut controller = Joy2L::new();
    
    // Get peripheral and subscribe to notifications
    let peripheral = connection.peripheral();
    let mut notification_stream = peripheral.notifications().await?;
    
    // Track previous state for change detection
    let mut prev_buttons = String::new();
    let mut prev_stick = (0.0f32, 0.0f32);
    const STICK_DEADZONE: f32 = 0.15;
    
    println!("Listening for input... (Press Ctrl+C to exit)\n");
    
    // Process notifications
    while let Some(notification) = notification_stream.next().await {
        // Update controller state
        controller.update(&notification.value);
        
        // Detect button changes
        let current_buttons = format_buttons(&controller);
        if current_buttons != prev_buttons && !current_buttons.is_empty() {
            println!("🎮 Buttons: {}", current_buttons);
            prev_buttons = current_buttons;
        } else if current_buttons.is_empty() && !prev_buttons.is_empty() {
            println!("🎮 Buttons: (none)");
            prev_buttons.clear();
        }
        
        // Detect stick changes (with deadzone)
        let stick_x = controller.analog_stick.x;
        let stick_y = controller.analog_stick.y;
        
        let x_changed = (stick_x - prev_stick.0).abs() > 0.05;
        let y_changed = (stick_y - prev_stick.1).abs() > 0.05;
        
        if x_changed || y_changed {
            // Apply deadzone for display
            let display_x = if stick_x.abs() < STICK_DEADZONE { 0.0 } else { stick_x };
            let display_y = if stick_y.abs() < STICK_DEADZONE { 0.0 } else { stick_y };
            
            if display_x.abs() > 0.01 || display_y.abs() > 0.01 {
                println!("🕹️  Stick: X={:+.2}, Y={:+.2} {}", 
                    display_x, display_y, format_stick_direction(display_x, display_y));
            } else if prev_stick.0.abs() > STICK_DEADZONE || prev_stick.1.abs() > STICK_DEADZONE {
                println!("🕹️  Stick: centered");
            }
            
            prev_stick = (stick_x, stick_y);
        }
    }
    
    Ok(())
}

/// Format pressed buttons as a string
fn format_buttons(controller: &Joy2L) -> String {
    let mut buttons = Vec::new();
    
    if controller.buttons.zl { buttons.push("ZL"); }
    if controller.buttons.l { buttons.push("L"); }
    if controller.buttons.minus { buttons.push("−"); }
    if controller.buttons.capture { buttons.push("📷"); }
    if controller.buttons.l3 { buttons.push("L3"); }
    if controller.buttons.up { buttons.push("↑"); }
    if controller.buttons.down { buttons.push("↓"); }
    if controller.buttons.left { buttons.push("←"); }
    if controller.buttons.right { buttons.push("→"); }
    if controller.buttons.sll { buttons.push("SL"); }
    if controller.buttons.srl { buttons.push("SR"); }
    
    buttons.join(" + ")
}

/// Format stick direction as arrows
fn format_stick_direction(x: f32, y: f32) -> String {
    if x.abs() < 0.01 && y.abs() < 0.01 {
        return String::new();
    }
    
    let mut dir = Vec::new();
    
    if y > 0.3 { dir.push("↑"); }
    if y < -0.3 { dir.push("↓"); }
    if x < -0.3 { dir.push("←"); }
    if x > 0.3 { dir.push("→"); }
    
    if dir.is_empty() {
        String::new()
    } else {
        format!("[{}]", dir.join(""))
    }
}
