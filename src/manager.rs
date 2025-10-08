//! High-level Joy-Con 2 Manager
//!
//! This module provides a high-level interface for managing Joy-Con 2 controllers,
//! handling connection, event forwarding, and executor integration.

use crate::backend::{KeyboardBackend, MouseBackend};
use crate::joycon2::connection::{JoyConConnection, Side};
use crate::joycon2::controller::{Joy2L, Joy2R};
use crate::joycon2::mac_cache::ControllerCache;
use crate::mapping::config::{ButtonType, Config, ControllerSide, JoyConEvent, StickType};
use crate::mapping::executor::MappingExecutor;
use btleplug::api::Peripheral as _;
use btleplug::platform::Peripheral;
use crossbeam_channel::{bounded, Receiver, Sender};
use futures::stream::StreamExt;
use log::{debug, info, warn};
use std::collections::HashSet;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Runtime;

/// Manager for handling Joy-Con 2 controllers
pub struct JoyConManager<K, M>
where
    K: KeyboardBackend + Clone + Send + 'static,
    M: MouseBackend + Clone + Send + 'static,
{
    config: Config,
    keyboard: K,
    mouse: M,
    event_sender: Sender<JoyConEvent>,
    event_receiver: Receiver<JoyConEvent>,
    /// Running flag
    running: Arc<AtomicBool>,
    /// Track MAC addresses of connected controllers to avoid duplicates
    connected_macs: Arc<Mutex<HashSet<String>>>,
    /// Controller cache for quick reconnection
    mac_cache: Arc<Mutex<ControllerCache>>,
    /// Channel to send discovered peripherals to controller threads
    peripheral_sender: Sender<(Peripheral, Side, String)>,
    peripheral_receiver: Receiver<(Peripheral, Side, String)>,
}

impl<K, M> JoyConManager<K, M>
where
    K: KeyboardBackend + Clone + Send + 'static,
    M: MouseBackend + Clone + Send + 'static,
{
    /// Create a new Joy-Con manager
    pub fn new(config: Config, keyboard: K, mouse: M) -> Self {
        let (event_sender, event_receiver) = bounded(100);
        let (peripheral_sender, peripheral_receiver) = bounded(10);
        
        // Load MAC cache from disk
        let mac_cache = ControllerCache::load();
        info!("Loaded {} cached controllers", mac_cache.len());
        
        Self {
            config,
            keyboard,
            mouse,
            event_sender,
            event_receiver,
            running: Arc::new(AtomicBool::new(false)),
            connected_macs: Arc::new(Mutex::new(HashSet::new())),
            mac_cache: Arc::new(Mutex::new(mac_cache)),
            peripheral_sender,
            peripheral_receiver,
        }
    }
    
    /// Start the manager - scans for controllers and starts event processing
    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        if self.running.load(Ordering::SeqCst) {
            return Err("Manager is already running".into());
        }
        
        self.running.store(true, Ordering::SeqCst);
        
        info!("Starting Joy-Con Manager...");
        
        // Start executor thread
        self.start_executor_thread();
        
        // Start single scan thread that finds both controllers
        info!("Starting controller scanner...");
        self.start_scan_thread()?;
        
        // Start controller handler threads (one for each side)
        info!("Starting controller handlers...");
        self.start_controller_thread(Side::Left)?;
        self.start_controller_thread(Side::Right)?;
        
        info!("✓ Manager started! Scanning for controllers...");
        info!("  Press the sync button on your Joy-Cons");
        
        Ok(())
    }
    
    /// Stop the manager
    pub fn stop(&mut self) {
        info!("Stopping Joy-Con Manager...");
        self.running.store(false, Ordering::SeqCst);
    }
    
    /// Check if the manager is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// Get the event receiver (for external event processing)
    pub fn get_event_receiver(&self) -> &Receiver<JoyConEvent> {
        &self.event_receiver
    }
    
    /// Start the scanner thread that finds both Left and Right controllers
    fn start_scan_thread(&self) -> Result<(), Box<dyn Error>> {
        let peripheral_sender = self.peripheral_sender.clone();
        let running = Arc::clone(&self.running);
        let connected_macs = Arc::clone(&self.connected_macs);
        let mac_cache = Arc::clone(&self.mac_cache);
        
        thread::Builder::new()
            .name("scanner".to_string())
            .spawn(move || {
                let rt = Runtime::new().expect("Failed to create tokio runtime");
                
                rt.block_on(async {
                    info!("Scanner thread started");
                    
                    while running.load(Ordering::SeqCst) {
                        match Self::scan_for_controllers(
                            peripheral_sender.clone(),
                            running.clone(),
                            connected_macs.clone(),
                            mac_cache.clone()
                        ).await {
                            Ok(_) => {
                                debug!("Scan cycle completed");
                            }
                            Err(e) => {
                                warn!("Scan error: {}, retrying in 5 seconds...", e);
                                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                            }
                        }
                    }
                    
                    info!("Scanner thread exited");
                });
            })?;
        
        Ok(())
    }
    
    /// Scan for Joy-Con controllers and send discovered ones to the handler threads
    async fn scan_for_controllers(
        peripheral_sender: Sender<(Peripheral, Side, String)>,
        running: Arc<AtomicBool>,
        connected_macs: Arc<Mutex<HashSet<String>>>,
        mac_cache: Arc<Mutex<ControllerCache>>,
    ) -> Result<(), Box<dyn Error>> {
        use btleplug::api::{Central, Manager as _, CentralEvent};
        use btleplug::platform::Manager;
        use crate::joycon2::constants::{NINTENDO_COMPANY_ID, JOYCON_DATA_PREFIX};
        
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        
        if adapters.is_empty() {
            return Err("No Bluetooth adapters found".into());
        }
        
        let adapter = adapters.into_iter().next().unwrap();
        adapter.start_scan(Default::default()).await?;
        
        let mut events = adapter.events().await?;
        
        // Scan for Joy-Con controllers
        while running.load(Ordering::SeqCst) {
            tokio::select! {
                Some(event) = events.next() => {
                    if let CentralEvent::ManufacturerDataAdvertisement { id, manufacturer_data } = event {
                        // Check if manufacturer data contains Nintendo company ID
                        if let Some(data) = manufacturer_data.get(&NINTENDO_COMPANY_ID) {
                            // Check if data starts with Joy-Con prefix
                            if data.len() >= JOYCON_DATA_PREFIX.len()
                                && data[..JOYCON_DATA_PREFIX.len()] == JOYCON_DATA_PREFIX
                            {
                                // Determine side from byte 5
                                if data.len() >= 6 {
                                    let side_byte = data[5];
                                    
                                    let side = match side_byte {
                                        0x67 => Some(Side::Left),
                                        0x66 => Some(Side::Right),
                                        _ => None,
                                    };
                                    
                                    if let Some(side) = side {
                                        let peripheral = adapter.peripheral(&id).await?;
                                        let properties = peripheral.properties().await?.unwrap();
                                        let mac_address = properties.address.to_string();
                                        
                                        // Check if already connected
                                        {
                                            let macs = connected_macs.lock().unwrap();
                                            if macs.contains(&mac_address) {
                                                continue; // Skip already connected controller
                                            }
                                        }
                                        
                                        let name = properties.local_name.unwrap_or_else(|| "Unknown".to_string());
                                        
                                        info!("✓ Found {:?} Joy-Con: {} ({})", side, name, mac_address);
                                        
                                        // Send to appropriate handler thread
                                        let _ = peripheral_sender.send((peripheral, side, mac_address.clone()));
                                        
                                        // Cache this controller
                                        {
                                            let mut cache = mac_cache.lock().unwrap();
                                            cache.add_controller(mac_address, side, Some(name));
                                            let _ = cache.save();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                    // Periodic check
                }
            }
        }
        
        adapter.stop_scan().await?;
        Ok(())
    }
    
    /// Start the executor thread
    fn start_executor_thread(&self) {
        let receiver = self.event_receiver.clone();
        let keyboard = self.keyboard.clone();
        let mouse = self.mouse.clone();
        let config = self.config.clone();
        let running = Arc::clone(&self.running);
        
        thread::Builder::new()
            .name("executor".to_string())
            .spawn(move || {
                info!("Executor thread started");
                
                let mut executor = MappingExecutor::new(config, keyboard, mouse);
                
                while running.load(Ordering::SeqCst) {
                    match receiver.recv_timeout(std::time::Duration::from_millis(16)) {
                        Ok(event) => {
                            executor.process_event(&event);
                        }
                        Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                            // No event, but update continuous movements (stick held positions)
                            // This runs at ~60Hz (every 16ms) to keep mouse moving smoothly
                        }
                        Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                            warn!("Event channel disconnected");
                            break;
                        }
                    }
                    
                    // Always update continuous movements on each loop iteration
                    // This ensures smooth mouse movement when stick is held
                    executor.update_continuous_movements();
                }
                
                info!("Executor thread stopped");
            })
            .expect("Failed to spawn executor thread");
    }
    
    /// Start a controller thread for the given side
    /// This thread waits for peripherals from the scanner thread
    fn start_controller_thread(&self, side: Side) -> Result<(), Box<dyn Error>> {
        let sender = self.event_sender.clone();
        let running = Arc::clone(&self.running);
        let connected_macs = Arc::clone(&self.connected_macs);
        let peripheral_receiver = self.peripheral_receiver.clone();
        
        let thread_name = match side {
            Side::Left => "controller-left",
            Side::Right => "controller-right",
        };
        
        thread::Builder::new()
            .name(thread_name.to_string())
            .spawn(move || {
                let rt = Runtime::new().expect("Failed to create tokio runtime");
                
                rt.block_on(async {
                    info!("Controller {:?} handler started, waiting for peripheral...", side);
                    
                    while running.load(Ordering::SeqCst) {
                        // Wait for a peripheral from the scanner
                        match peripheral_receiver.recv_timeout(std::time::Duration::from_secs(1)) {
                            Ok((peripheral, discovered_side, mac_address)) => {
                                // Only handle peripherals for our side
                                if discovered_side != side {
                                    continue;
                                }
                                
                                info!("Handling {:?} controller: {}", side, mac_address);
                                
                                match Self::controller_loop(
                                    peripheral,
                                    side,
                                    mac_address.clone(),
                                    sender.clone(),
                                    running.clone(),
                                    connected_macs.clone()
                                ).await {
                                    Ok(_) => {
                                        info!("Controller {:?} disconnected", side);
                                    }
                                    Err(e) => {
                                        warn!("Controller {:?} error: {}", side, e);
                                    }
                                }
                            }
                            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                                // No peripheral yet, continue waiting
                                continue;
                            }
                            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                                warn!("Peripheral channel disconnected");
                                break;
                            }
                        }
                    }
                    
                    info!("Controller {:?} handler exited", side);
                });
            })?;
        
        Ok(())
    }
    
    /// Main controller loop (runs in async context)
    async fn controller_loop(
        peripheral: Peripheral,
        side: Side,
        mac_address: String,
        sender: Sender<JoyConEvent>,
        running: Arc<AtomicBool>,
        connected_macs: Arc<Mutex<HashSet<String>>>,
    ) -> Result<(), Box<dyn Error>> {
        let controller_side = match side {
            Side::Left => ControllerSide::Left,
            Side::Right => ControllerSide::Right,
        };
        
        // Check if this MAC is already connected
        {
            let mut macs = connected_macs.lock().unwrap();
            if macs.contains(&mac_address) {
                return Err(format!("Controller {} already connected to another side", mac_address).into());
            }
            // Register this MAC
            macs.insert(mac_address.clone());
        }
        
        // Create connection and initialize
        let mut connection = JoyConConnection::new(peripheral, side);
        
        info!("Connecting to {:?} controller ({})", side, mac_address);
        connection.connect().await?;
        connection.initialize().await?;
        
        info!("✓ Controller {:?} ready! (MAC: {})", side, mac_address);
        
        // Send connected event
        let _ = sender.send(JoyConEvent::Connected { side: controller_side });
        
        // Get peripheral and notification stream
        let peripheral = connection.peripheral();
        let mut notification_stream = peripheral.notifications().await?;
        
        // Create controller state tracker
        match side {
            Side::Left => {
                let mut controller = Joy2L::new();
                let mut prev_buttons = create_left_button_snapshot(&controller);
                let mut prev_stick = (0.0f32, 0.0f32);
                let mut prev_gyro = (0.0f32, 0.0f32, 0.0f32);
                let mut battery_logged = false;
                
                while running.load(Ordering::SeqCst) {
                    tokio::select! {
                        Some(notification) = notification_stream.next() => {
                            controller.update(&notification.value);
                            
                            // Log battery level once after first update
                            if !battery_logged {
                                info!("  Battery Level: {:.0}%", controller.battery_level);
                                battery_logged = true;
                            }
                            
                            // Check for button changes
                            Self::process_left_button_events(&controller, &mut prev_buttons, &sender);
                            
                            // Check for stick changes
                            let stick_x = controller.analog_stick.x;
                            let stick_y = controller.analog_stick.y;
                            
                            if (stick_x - prev_stick.0).abs() > 0.05 || (stick_y - prev_stick.1).abs() > 0.05 {
                                let _ = sender.send(JoyConEvent::StickMoved {
                                    stick: StickType::Left,
                                    x: stick_x,
                                    y: stick_y,
                                });
                                prev_stick = (stick_x, stick_y);
                            }
                            
                            // Check for gyro changes
                            let gyro_x = controller.gyroscope.x;
                            let gyro_y = controller.gyroscope.y;
                            let gyro_z = controller.gyroscope.z;
                            
                            if (gyro_x - prev_gyro.0).abs() > 0.5 
                                || (gyro_y - prev_gyro.1).abs() > 0.5 
                                || (gyro_z - prev_gyro.2).abs() > 0.5 {
                                let _ = sender.send(JoyConEvent::GyroUpdate {
                                    side: controller_side,
                                    x: gyro_x,
                                    y: gyro_y,
                                    z: gyro_z,
                                });
                                prev_gyro = (gyro_x, gyro_y, gyro_z);
                            }
                        }
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(1)) => {
                            // Timeout check
                            if !running.load(Ordering::SeqCst) {
                                break;
                            }
                        }
                    }
                }
            }
            
            Side::Right => {
                let mut controller = Joy2R::new();
                let mut prev_buttons = create_right_button_snapshot(&controller);
                let mut prev_stick = (0.0f32, 0.0f32);
                let mut prev_gyro = (0.0f32, 0.0f32, 0.0f32);
                let mut battery_logged = false;
                
                while running.load(Ordering::SeqCst) {
                    tokio::select! {
                        Some(notification) = notification_stream.next() => {
                            controller.update(&notification.value);
                            
                            // Log battery level once after first update
                            if !battery_logged {
                                info!("  Battery Level: {:.0}%", controller.battery_level);
                                battery_logged = true;
                            }
                            
                            // Check for button changes
                            Self::process_right_button_events(&controller, &mut prev_buttons, &sender);
                            
                            // Check for stick changes
                            let stick_x = controller.analog_stick.x;
                            let stick_y = controller.analog_stick.y;
                            
                            if (stick_x - prev_stick.0).abs() > 0.05 || (stick_y - prev_stick.1).abs() > 0.05 {
                                let _ = sender.send(JoyConEvent::StickMoved {
                                    stick: StickType::Right,
                                    x: stick_x,
                                    y: stick_y,
                                });
                                prev_stick = (stick_x, stick_y);
                            }
                            
                            // Check for gyro changes
                            let gyro_x = controller.gyroscope.x;
                            let gyro_y = controller.gyroscope.y;
                            let gyro_z = controller.gyroscope.z;
                            
                            if (gyro_x - prev_gyro.0).abs() > 0.5 
                                || (gyro_y - prev_gyro.1).abs() > 0.5 
                                || (gyro_z - prev_gyro.2).abs() > 0.5 {
                                let _ = sender.send(JoyConEvent::GyroUpdate {
                                    side: controller_side,
                                    x: gyro_x,
                                    y: gyro_y,
                                    z: gyro_z,
                                });
                                prev_gyro = (gyro_x, gyro_y, gyro_z);
                            }
                        }
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(1)) => {
                            // Timeout check
                            if !running.load(Ordering::SeqCst) {
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        // Explicitly disconnect before dropping the connection
        info!("Disconnecting {:?} controller...", side);
        if let Err(e) = connection.disconnect().await {
            warn!("Error disconnecting {:?} controller: {}", side, e);
        }
        
        // Remove MAC from connected set
        {
            let mut macs = connected_macs.lock().unwrap();
            macs.remove(&mac_address);
            info!("Controller {:?} (MAC: {}) removed from tracking", side, mac_address);
        }
        
        // Send disconnected event
        let _ = sender.send(JoyConEvent::Disconnected { side: controller_side });
        
        Ok(())
    }
    
    /// Process left controller button events
    fn process_left_button_events(
        controller: &Joy2L,
        prev_buttons: &mut LeftButtonSnapshot,
        sender: &Sender<JoyConEvent>,
    ) {
        let buttons = &controller.buttons;
        
        // Check each button for changes
        Self::check_button_change(buttons.zl, &mut prev_buttons.zl, ButtonType::ZL, sender);
        Self::check_button_change(buttons.l, &mut prev_buttons.l, ButtonType::L, sender);
        Self::check_button_change(buttons.minus, &mut prev_buttons.minus, ButtonType::Minus, sender);
        Self::check_button_change(buttons.capture, &mut prev_buttons.capture, ButtonType::Capture, sender);
        Self::check_button_change(buttons.l3, &mut prev_buttons.l3, ButtonType::LeftStickClick, sender);
        Self::check_button_change(buttons.up, &mut prev_buttons.up, ButtonType::DpadUp, sender);
        Self::check_button_change(buttons.down, &mut prev_buttons.down, ButtonType::DpadDown, sender);
        Self::check_button_change(buttons.left, &mut prev_buttons.left, ButtonType::DpadLeft, sender);
        Self::check_button_change(buttons.right, &mut prev_buttons.right, ButtonType::DpadRight, sender);
        Self::check_button_change(buttons.sll, &mut prev_buttons.sll, ButtonType::SLL, sender);
        Self::check_button_change(buttons.srl, &mut prev_buttons.srl, ButtonType::SRL, sender);
    }
    
    /// Process right controller button events
    fn process_right_button_events(
        controller: &Joy2R,
        prev_buttons: &mut RightButtonSnapshot,
        sender: &Sender<JoyConEvent>,
    ) {
        let buttons = &controller.buttons;
        
        // Check each button for changes
        Self::check_button_change(buttons.a, &mut prev_buttons.a, ButtonType::A, sender);
        Self::check_button_change(buttons.b, &mut prev_buttons.b, ButtonType::B, sender);
        Self::check_button_change(buttons.x, &mut prev_buttons.x, ButtonType::X, sender);
        Self::check_button_change(buttons.y, &mut prev_buttons.y, ButtonType::Y, sender);
        Self::check_button_change(buttons.r, &mut prev_buttons.r, ButtonType::R, sender);
        Self::check_button_change(buttons.zr, &mut prev_buttons.zr, ButtonType::ZR, sender);
        Self::check_button_change(buttons.plus, &mut prev_buttons.plus, ButtonType::Plus, sender);
        Self::check_button_change(buttons.home, &mut prev_buttons.home, ButtonType::Home, sender);
        Self::check_button_change(buttons.r3, &mut prev_buttons.r3, ButtonType::RightStickClick, sender);
        Self::check_button_change(buttons.slr, &mut prev_buttons.slr, ButtonType::SLR, sender);
        Self::check_button_change(buttons.srr, &mut prev_buttons.srr, ButtonType::SRR, sender);
        Self::check_button_change(buttons.chat, &mut prev_buttons.chat, ButtonType::Chat, sender);
    }
    
    /// Check if a button state changed and send appropriate event
    fn check_button_change(
        current: bool,
        previous: &mut bool,
        button_type: ButtonType,
        sender: &Sender<JoyConEvent>,
    ) {
        if current && !*previous {
            let _ = sender.send(JoyConEvent::ButtonPressed(button_type));
            *previous = true;
        } else if !current && *previous {
            let _ = sender.send(JoyConEvent::ButtonReleased(button_type));
            *previous = false;
        }
    }
}

/// Implement Drop to gracefully shutdown and disconnect controllers
impl<K, M> Drop for JoyConManager<K, M>
where
    K: KeyboardBackend + Clone + Send + 'static,
    M: MouseBackend + Clone + Send + 'static,
{
    fn drop(&mut self) {
        // Always attempt cleanup, regardless of running state
        let was_running = self.running.swap(false, Ordering::SeqCst);
        
        if was_running {
            info!("Shutting down Joy-Con Manager (Drop trait)...");
            
            // Clear connected MACs to allow reconnection
            {
                let mut macs = self.connected_macs.lock().unwrap();
                macs.clear();
            }
            
            // Give threads time to detect the running flag change and clean up
            // The controller loops will exit, which will drop their JoyConConnection
            // instances, triggering proper Bluetooth disconnection
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            info!("✓ Joy-Con Manager shutdown complete");
        }
    }
}

/// Snapshot of left controller button states
struct LeftButtonSnapshot {
    zl: bool,
    l: bool,
    minus: bool,
    capture: bool,
    l3: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    sll: bool,
    srl: bool,
}

/// Snapshot of right controller button states
struct RightButtonSnapshot {
    a: bool,
    b: bool,
    x: bool,
    y: bool,
    r: bool,
    zr: bool,
    plus: bool,
    home: bool,
    r3: bool,
    slr: bool,
    srr: bool,
    chat: bool,
}

/// Create a snapshot of left controller button states
fn create_left_button_snapshot(controller: &Joy2L) -> LeftButtonSnapshot {
    LeftButtonSnapshot {
        zl: controller.buttons.zl,
        l: controller.buttons.l,
        minus: controller.buttons.minus,
        capture: controller.buttons.capture,
        l3: controller.buttons.l3,
        up: controller.buttons.up,
        down: controller.buttons.down,
        left: controller.buttons.left,
        right: controller.buttons.right,
        sll: controller.buttons.sll,
        srl: controller.buttons.srl,
    }
}

/// Create a snapshot of right controller button states
fn create_right_button_snapshot(controller: &Joy2R) -> RightButtonSnapshot {
    RightButtonSnapshot {
        a: controller.buttons.a,
        b: controller.buttons.b,
        x: controller.buttons.x,
        y: controller.buttons.y,
        r: controller.buttons.r,
        zr: controller.buttons.zr,
        plus: controller.buttons.plus,
        home: controller.buttons.home,
        r3: controller.buttons.r3,
        slr: controller.buttons.slr,
        srr: controller.buttons.srr,
        chat: controller.buttons.chat,
    }
}
