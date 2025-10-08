//! Joy-Con 2 protocol constants
//!
//! This module contains all the constants needed for Joy-Con 2 communication:
//! - BLE UUIDs and manufacturer data
//! - Protocol commands and subcommands
//! - SPI flash memory addresses
//! - Input report IDs

use uuid::Uuid;

// ============================================================================
// BLE Discovery Constants
// ============================================================================

/// Nintendo Co., Ltd. company ID for BLE manufacturer data
pub const NINTENDO_COMPANY_ID: u16 = 0x0553;

/// Manufacturer data prefix for Joy-Con 2 controllers
pub const JOYCON_DATA_PREFIX: [u8; 5] = [0x01, 0x00, 0x03, 0x7e, 0x05];

// ============================================================================
// BLE Service & Characteristic UUIDs
// ============================================================================

/// Main Joy-Con 2 service UUID
pub const JOYCON2_SERVICE_UUID: Uuid = Uuid::from_u128(0xab7de9be_89fe_49ad_828f_118f09df7fd0);

/// TX characteristic UUID (controller -> host, for input data)
/// This is the NOTIFY characteristic we read input reports from
pub const TX_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0xab7de9be_89fe_49ad_828f_118f09df7fd2);

/// Command characteristic UUID (host -> controller, for commands)
/// This is the WRITE characteristic we send commands to (Joy-Con 2 specific)
pub const CMD_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x649d4ac9_8eb7_4e6c_af44_1ea54fe5f005);

/// Command response characteristic UUID (controller -> host, for command acknowledgments)
/// This is the NOTIFY characteristic we subscribe to for command responses
pub const CMD_RESPONSE_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0xc765a961_d9d8_4d36_a20a_5315b111836a);

// ============================================================================
// Joy-Con 2 Specific Commands (hex format)
// ============================================================================

/// Connected vibration command (feedback when controller connects)
pub const JOY2_CONNECTED_VIBRATION: &[u8] = &[0x0A, 0x91, 0x01, 0x02, 0x00, 0x04, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00];

/// Set player LED command (complete 16-byte command)
/// Byte 8 (index 8) is the LED value: 1=LED1, 2=LED2, 4=LED3, 8=LED4, combinations possible
/// Format: 09 91 00 07 00 08 00 00 0X 00 00 00 00 00 00 00
pub const JOY2_SET_PLAYER_LED_TEMPLATE: [u8; 16] = [0x09, 0x91, 0x00, 0x07, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
pub const JOY2_LED_VALUE_INDEX: usize = 8;

/// Initialize sensor data (IMU setup) - Step 1
/// From Joy2Win research: The order and data (0x2F) is very important!
pub const JOY2_INIT_SENSOR_DATA: &[u8] = &[0x0C, 0x91, 0x01, 0x02, 0x00, 0x04, 0x00, 0x00, 0x2F, 0x00, 0x00, 0x00];

/// Finalize sensor data - Step 2
pub const JOY2_FINALIZE_SENSOR_DATA: &[u8] = &[0x0C, 0x91, 0x01, 0x03, 0x00, 0x04, 0x00, 0x00, 0x2F, 0x00, 0x00, 0x00];

/// Start sensor data streaming - Step 3
pub const JOY2_START_SENSOR_DATA: &[u8] = &[0x0C, 0x91, 0x01, 0x04, 0x00, 0x04, 0x00, 0x00, 0x2F, 0x00, 0x00, 0x00];

/// Save MAC address step 1 (format: prefix + mac_addr1(12 hex chars) + mac_addr2(12 hex chars))
pub const JOY2_SAVE_MAC_ADDR_STEP1_PREFIX: &[u8] = &[0x15, 0x91, 0x01, 0x01, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x02];

/// Save MAC address step 2
pub const JOY2_SAVE_MAC_ADDR_STEP2: &[u8] = &[0x15, 0x91, 0x01, 0x04, 0x00, 0x11, 0x00, 0x00, 0x00, 0x08, 0x06, 0x5A, 0x60, 0xE9, 0x02, 0xE4, 0xE1, 0x02, 0x02, 0x9E, 0x3F, 0xA3, 0x9A, 0x78, 0xD1];

/// Save MAC address step 3
pub const JOY2_SAVE_MAC_ADDR_STEP3: &[u8] = &[0x15, 0x91, 0x01, 0x02, 0x00, 0x11, 0x00, 0x00, 0x00, 0x93, 0x4E, 0x58, 0x0F, 0x16, 0x3A, 0xEE, 0xCF, 0xB5, 0x75, 0xFC, 0x91, 0x36, 0xB2, 0x2F, 0xBB];

/// Save MAC address step 4
pub const JOY2_SAVE_MAC_ADDR_STEP4: &[u8] = &[0x15, 0x91, 0x01, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00];

// ============================================================================
// Timing Constants
// ============================================================================

/// Delay between commands (milliseconds)
pub const COMMAND_DELAY_MS: u64 = 50;
