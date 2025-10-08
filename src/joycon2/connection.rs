//! Joy-Con connection management
//!
//! This module handles the Bluetooth connection to the Joy-Con controllers,
//! including pairing, input reporting, and disconnection.

use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, WriteType};
use btleplug::platform::{Manager, Peripheral};
use futures::stream::StreamExt;
use log::{debug, info};
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

use crate::joycon2::constants::*;

/// Controller side/type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Initializing,
    Ready,
}

/// Joy-Con BLE connection wrapper
pub struct JoyConConnection {
    peripheral: Peripheral,
    side: Side,
    state: ConnectionState,
    
    // Characteristics
    tx_char: Option<Characteristic>,  // Input data notifications
    cmd_char: Option<Characteristic>,  // Send commands
    cmd_response_char: Option<Characteristic>,  // Command responses
    
    // Optional MAC address for pairing (Joy-Con 2 specific)
    mac_address: Option<[u8; 6]>,
}

impl JoyConConnection {
    /// Create a new Joy-Con connection from a peripheral
    pub fn new(peripheral: Peripheral, side: Side) -> Self {
        Self {
            peripheral,
            side,
            state: ConnectionState::Disconnected,
            tx_char: None,
            cmd_char: None,
            cmd_response_char: None,
            mac_address: None,
        }
    }
    
    /// Set MAC address for pairing (Joy-Con 2 specific, optional)
    pub fn set_mac_address(&mut self, mac_address: [u8; 6]) {
        self.mac_address = Some(mac_address);
    }
    
    /// Scan for Joy-Con controllers with side filtering
    /// 
    /// This will only return a controller that matches the requested side,
    /// preventing race conditions where multiple threads try to connect to the same controller.
    pub async fn scan(expected_side: Side) -> Result<Peripheral, Box<dyn Error>> {
        info!("Scanning for Joy-Con controllers...");
        
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        
        if adapters.is_empty() {
            return Err("No Bluetooth adapters found".into());
        }
        
        let adapter = adapters.into_iter().next().unwrap();
        
        // Start scanning
        adapter.start_scan(Default::default()).await?;
        
        let mut events = adapter.events().await?;
        
        // Expected byte value for the requested side
        let expected_byte = match expected_side {
            Side::Left => 0x67,
            Side::Right => 0x66,
        };
        
        // Scan for Joy-Con controller
        while let Some(event) = events.next().await {
            if let btleplug::api::CentralEvent::ManufacturerDataAdvertisement {
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
                        // Check if this is the correct side (byte 5)
                        if data.len() >= 6 {
                            let side_byte = data[5];
                            
                            // Only accept controllers that match the expected side
                            if side_byte != expected_byte {
                                debug!("Skipping controller with side byte 0x{:02x} (expected 0x{:02x})", side_byte, expected_byte);
                                continue;
                            }
                            
                            let peripheral = adapter.peripheral(&id).await?;
                            let properties = peripheral.properties().await?.unwrap();
                            let address = properties.address;
                            let name = properties.local_name.unwrap_or_else(|| "Unknown".to_string());
                            
                            let detected_side_name = match side_byte {
                                0x67 => "Left (0x67)",
                                0x66 => "Right (0x66)",
                                0x73 => "GCCon (0x73)",
                                byte => {
                                    debug!("Unknown device type: 0x{:02x}", byte);
                                    "Unknown"
                                }
                            };
                            
                            info!("✓ Controller found!");
                            info!("  Address: {}", address);
                            info!("  Name: {}", name);
                            info!("  Detected side: {}", detected_side_name);
                            
                            adapter.stop_scan().await?;
                            return Ok(peripheral);
                        }
                    }
                }
            }
        }
        
        Err("No Joy-Con controller found".into())
    }
    
    /// Connect to the Joy-Con
    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        self.state = ConnectionState::Connecting;
        
        info!("Connecting to Joy-Con...");
        self.peripheral.connect().await?;
        
        info!("Discovering services...");
        self.peripheral.discover_services().await?;
        
        // Find the Joy-Con 2 characteristics
        let characteristics = self.peripheral.characteristics();
        
        for char in characteristics {
            if char.uuid == TX_CHARACTERISTIC_UUID {
                self.tx_char = Some(char.clone());
                debug!("Found TX characteristic (input data)");
            } else if char.uuid == CMD_CHARACTERISTIC_UUID {
                self.cmd_char = Some(char.clone());
                debug!("Found CMD characteristic (send commands)");
            } else if char.uuid == CMD_RESPONSE_CHARACTERISTIC_UUID {
                self.cmd_response_char = Some(char.clone());
                debug!("Found CMD_RESPONSE characteristic (command acks)");
            }
        }
        
        if self.tx_char.is_none() || self.cmd_char.is_none() || self.cmd_response_char.is_none() {
            return Err("Failed to find required characteristics".into());
        }
        
        info!("✓ Connected successfully!");
        Ok(())
    }
    
    /// Initialize the Joy-Con (handshake process)
    /// This sends initialization commands and sets up the controller for data streaming
    pub async fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        self.state = ConnectionState::Initializing;
        
        info!("Initializing Joy-Con...");
        
        // Subscribe to command response notifications first
        if let Some(cmd_response_char) = &self.cmd_response_char {
            self.peripheral.subscribe(cmd_response_char).await?;
            debug!("Subscribed to CMD_RESPONSE notifications");
        }
        
        // Send initialization commands
        self.send_initialization_commands().await?;
        
        // Subscribe to TX notifications for input data
        if let Some(tx_char) = &self.tx_char {
            self.peripheral.subscribe(tx_char).await?;
            debug!("Subscribed to TX notifications (input data)");
        }
        
        self.state = ConnectionState::Ready;
        info!("✓ Joy-Con initialized and ready!");
        
        Ok(())
    }
    
    /// Send initialization commands to the controller
    async fn send_initialization_commands(&mut self) -> Result<(), Box<dyn Error>> {
        // Joy-Con 2 specific initialization sequence
        // Based on Joy2Win Python implementation
        
        // 0. Save MAC address (optional, for pairing with Switch)
        if let Some(mac_addr) = self.mac_address {
            info!("  Saving MAC address for pairing...");
            self.save_mac_address(mac_addr).await?;
        }
        
        // 1. Connection vibration (feedback to user)
        info!("  Sending connection vibration...");
        self.send_connection_vibration().await?;
        
        // 2. Set player LED (default: LED 1 only)
        info!("  Setting player LED...");
        let mut led_command = JOY2_SET_PLAYER_LED_TEMPLATE;
        led_command[JOY2_LED_VALUE_INDEX] = 0x01;  // LED 1 only
        self.send_command(&led_command, true).await?;
        
        // 3. Initialize sensor data (IMU step 1)
        info!("  Initializing sensor data...");
        self.send_command(JOY2_INIT_SENSOR_DATA, true).await?;
        
        // 4. Finalize sensor data (IMU step 2)
        info!("  Finalizing sensor data...");
        self.send_command(JOY2_FINALIZE_SENSOR_DATA, true).await?;
        
        // 5. Start sensor data streaming (IMU step 3)
        info!("  Starting sensor data stream...");
        self.send_command(JOY2_START_SENSOR_DATA, true).await?;

        
        
        Ok(())
    }
    
    /// Send connection vibration (user feedback)
    async fn send_connection_vibration(&mut self) -> Result<(), Box<dyn Error>> {
        self.send_command(JOY2_CONNECTED_VIBRATION, true).await
    }
    
    /// Save MAC address for pairing (Joy-Con 2 specific)
    /// This allows the Joy-Con 2 to pair with a Nintendo Switch
    async fn save_mac_address(&mut self, mac_addr: [u8; 6]) -> Result<(), Box<dyn Error>> {
        // Calculate the two MAC addresses needed
        // mac_addr1 = original MAC address
        // mac_addr2 = first byte - 1, rest stays the same
        let mac_addr1 = mac_addr;
        let mut mac_addr2 = mac_addr;
        mac_addr2[0] = mac_addr2[0].wrapping_sub(1);
        
        info!("  MAC addresses: {:02X?} + {:02X?}", mac_addr1, mac_addr2);
        
        // Step 1: Save MAC addresses
        let mut step1_command = Vec::new();
        step1_command.extend_from_slice(JOY2_SAVE_MAC_ADDR_STEP1_PREFIX);
        step1_command.extend_from_slice(&mac_addr1);
        step1_command.extend_from_slice(&mac_addr2);
        self.send_command(&step1_command, true).await?;
        
        // Step 2-4: Additional pairing steps (exact commands from Joy2Win)
        self.send_command(JOY2_SAVE_MAC_ADDR_STEP2, true).await?;
        self.send_command(JOY2_SAVE_MAC_ADDR_STEP3, true).await?;
        self.send_command(JOY2_SAVE_MAC_ADDR_STEP4, true).await?;
        
        info!("  ✓ MAC address saved successfully");
        Ok(())
    }
    
    /// Send a command to the controller (Joy-Con 2 specific format)
    async fn send_command(&mut self, data: &[u8], wait_response: bool) -> Result<(), Box<dyn Error>> {
        if let Some(cmd_char) = &self.cmd_char {
            debug!("Sending command: {} bytes", data.len());
            
            self.peripheral.write(cmd_char, data, WriteType::WithoutResponse).await?;
            
            // TODO: If wait_response is true, we should wait for a notification on cmd_response_char
            // For now, just add a delay
            if wait_response {
                sleep(Duration::from_millis(COMMAND_DELAY_MS)).await;
            }
            
            Ok(())
        } else {
            Err("CMD characteristic not found".into())
        }
    }
    
    /// Disconnect from the Joy-Con
    pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Disconnecting from Joy-Con...");
        
        // Unsubscribe from notifications
        if let Some(tx_char) = &self.tx_char {
            let _ = self.peripheral.unsubscribe(tx_char).await;
        }
        if let Some(cmd_response_char) = &self.cmd_response_char {
            let _ = self.peripheral.unsubscribe(cmd_response_char).await;
        }
        
        self.peripheral.disconnect().await?;
        self.state = ConnectionState::Disconnected;
        
        info!("✓ Disconnected successfully!");
        Ok(())
    }
    
    /// Get connection state
    pub fn state(&self) -> ConnectionState {
        self.state
    }
    
    /// Check if connected
    pub async fn is_connected(&self) -> Result<bool, Box<dyn Error>> {
        Ok(self.peripheral.is_connected().await?)
    }
    
    /// Get the peripheral (for reading notifications)
    pub fn peripheral(&self) -> &Peripheral {
        &self.peripheral
    }
    
    /// Get controller side
    pub fn side(&self) -> Side {
        self.side
    }
    
    /// Detect controller side from manufacturer data
    /// 
    /// Byte 5 (index 5) of the manufacturer data indicates the device type:
    /// - 0x67 = Left Joy-Con
    /// - 0x66 = Right Joy-Con
    /// - 0x73 = GameCube Controller
    pub async fn detect_side_from_manufacturer_data(&self) -> Option<Side> {
        let properties = self.peripheral.properties().await.ok()??;
        
        if let Some(manufacturer_data) = properties.manufacturer_data.get(&NINTENDO_COMPANY_ID) {
            if manufacturer_data.len() >= 6 {
                let byte5 = manufacturer_data[5];
                match byte5 {
                    0x67 => Some(Side::Left),
                    0x66 => Some(Side::Right),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Initialize a controller (combines scan, connect, and initialize)
pub async fn init_controller(side: Side) -> Result<JoyConConnection, Box<dyn Error>> {
    info!("Scanning for Joy-Con {}, press the sync button...", match side {
        Side::Left => "Left",
        Side::Right => "Right",
    });
    
    // Scan for controller matching the requested side (filters by manufacturer data byte 5)
    let peripheral = JoyConConnection::scan(side).await?;
    
    // Create connection
    let mut connection = JoyConConnection::new(peripheral, side);
    
    // Connect
    connection.connect().await?;
    
    // Initialize (handshake)
    connection.initialize().await?;
    
    Ok(connection)
}
