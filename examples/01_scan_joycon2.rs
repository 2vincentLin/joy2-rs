use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use std::error::Error;

// Nintendo Co., Ltd. company ID
const NINTENDO_COMPANY_ID: u16 = 0x0553;

// Manufacturer data prefix for Joy-Con
const JOYCON_DATA_PREFIX: [u8; 5] = [0x01, 0x00, 0x03, 0x7e, 0x05];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Joy-Con 2 scanner...");
    
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    
    if adapters.is_empty() {
        eprintln!("No Bluetooth adapters found");
        return Ok(());
    }
    
    let adapter = adapters.into_iter().next().unwrap();
    println!("Using Bluetooth adapter: {}", adapter.adapter_info().await?);
    
    // Start scanning
    adapter.start_scan(ScanFilter::default()).await?;
    println!("Scanning for Joy-Con controllers...");
    
    let mut events = adapter.events().await?;
    
    // Keep scanning for Joy-Con controllers
    while let Some(event) = events.next().await {
        if let CentralEvent::ManufacturerDataAdvertisement {
            id,
            manufacturer_data,
        } = event
        {
            // Check if manufacturer data contains Nintendo company ID
            if let Some(data) = manufacturer_data.get(&NINTENDO_COMPANY_ID) {
                // Check if data starts with Joy-Con prefix
                if data.len() >= JOYCON_DATA_PREFIX.len()
                    && data[..JOYCON_DATA_PREFIX.len()] == JOYCON_DATA_PREFIX
                {
                    let peripheral = adapter.peripheral(&id).await?;
                    let properties = peripheral.properties().await?.unwrap();
                    let address = properties.address;
                    let name = properties.local_name.unwrap_or_else(|| "Unknown".to_string());
                    
                    println!("✓ Controller found!");
                    println!("  Address: {}", address);
                    println!("  Name: {}", name);
                    println!("  Manufacturer data: {:02x?}", data);
                    println!();
                }
            }
        }
    }
    
    Ok(())
}