use joy2_rs::joycon2::connection::{init_controller, Side};
use std::error::Error;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::builder().filter_level(log::LevelFilter::Info).init();
    
    println!("Joy-Con 2 Connection & Initialization Example");
    println!("=============================================\n");
    
    // Initialize a Joy-Con Left controller
    // This will: scan -> connect -> initialize (handshake)
    println!("Press the sync button on your Joy-Con Left now...\n");
    
    let mut connection = init_controller(Side::Left).await?;
    
    println!("\n✓ Joy-Con is fully initialized and ready to receive data!");
    println!("  State: {:?}", connection.state());
    println!("  Side: {:?}", connection.side());
    println!("  Connected: {}", connection.is_connected().await?);
    
    println!("\nKeeping connection alive for 10 seconds...");
    println!("(In the next step, we'll implement data reception and parsing)\n");
    
    sleep(Duration::from_secs(10)).await;
    
    // Disconnect
    println!("Disconnecting...");
    connection.disconnect().await?;
    
    println!("✓ Done!");
    
    Ok(())
}
