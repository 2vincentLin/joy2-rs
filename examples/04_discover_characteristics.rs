use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use std::error::Error;

const NINTENDO_COMPANY_ID: u16 = 0x0553;
const JOYCON_DATA_PREFIX: [u8; 5] = [0x01, 0x00, 0x03, 0x7e, 0x05];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    println!("Joy-Con 2 Characteristic Discovery");
    println!("==================================\n");
    
    println!("Press the sync button on your Joy-Con 2 now...\n");
    
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    
    if adapters.is_empty() {
        return Err("No Bluetooth adapters found".into());
    }
    
    let adapter = adapters.into_iter().next().unwrap();
    adapter.start_scan(ScanFilter::default()).await?;
    
    let mut events = adapter.events().await?;
    
    // Find Joy-Con
    let peripheral = loop {
        if let Some(event) = events.next().await {
            if let btleplug::api::CentralEvent::ManufacturerDataAdvertisement {
                id,
                manufacturer_data,
            } = event
            {
                if let Some(data) = manufacturer_data.get(&NINTENDO_COMPANY_ID) {
                    if data.len() >= JOYCON_DATA_PREFIX.len()
                        && data[..JOYCON_DATA_PREFIX.len()] == JOYCON_DATA_PREFIX
                    {
                        let peripheral = adapter.peripheral(&id).await?;
                        let properties = peripheral.properties().await?.unwrap();
                        println!("✓ Found Joy-Con!");
                        println!("  Address: {}", properties.address);
                        println!("  Name: {}\n", properties.local_name.unwrap_or_default());
                        
                        adapter.stop_scan().await?;
                        break peripheral;
                    }
                }
            }
        }
    };
    
    // Connect
    println!("Connecting...");
    peripheral.connect().await?;
    println!("✓ Connected\n");
    
    // Discover services
    println!("Discovering services and characteristics...\n");
    peripheral.discover_services().await?;
    
    // List all services
    let services = peripheral.services();
    println!("Found {} services:", services.len());
    for service in services {
        println!("  Service UUID: {}", service.uuid);
        println!("    Primary: {}", service.primary);
        
        // List characteristics
        for char in &service.characteristics {
            println!("    Characteristic UUID: {}", char.uuid);
            println!("      Properties: {:?}", char.properties);
        }
        println!();
    }
    
    // Disconnect
    peripheral.disconnect().await?;
    println!("✓ Disconnected");
    
    Ok(())
}
