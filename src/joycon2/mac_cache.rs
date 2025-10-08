//! MAC address cache for quick reconnection
//!
//! This module manages a local cache of previously seen Joy-Con controllers,
//! storing their MAC addresses and device types for faster reconnection.

use crate::joycon2::connection::Side;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Cache file name
const CACHE_FILENAME: &str = "joycon_cache.json";

/// Get the cache file path (in the same directory as the executable or current dir)
fn get_cache_path() -> PathBuf {
    // Try to use the executable directory first
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join(CACHE_FILENAME);
        }
    }
    
    // Fallback to current directory
    PathBuf::from(CACHE_FILENAME)
}

/// Cached controller information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedController {
    /// MAC address (string format like "AA:BB:CC:DD:EE:FF")
    pub mac_address: String,
    
    /// Controller side/type
    pub side: CachedSide,
    
    /// Optional friendly name
    #[serde(default)]
    pub name: Option<String>,
    
    /// Last seen timestamp (Unix timestamp)
    #[serde(default)]
    pub last_seen: u64,
}

/// Serializable version of Side enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CachedSide {
    Left,
    Right,
}

impl From<Side> for CachedSide {
    fn from(side: Side) -> Self {
        match side {
            Side::Left => CachedSide::Left,
            Side::Right => CachedSide::Right,
        }
    }
}

impl From<CachedSide> for Side {
    fn from(cached: CachedSide) -> Self {
        match cached {
            CachedSide::Left => Side::Left,
            CachedSide::Right => Side::Right,
        }
    }
}

/// Controller cache storage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ControllerCache {
    /// Map of MAC address -> controller info
    pub controllers: HashMap<String, CachedController>,
}

impl ControllerCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            controllers: HashMap::new(),
        }
    }
    
    /// Load cache from disk
    pub fn load() -> Self {
        let path = get_cache_path();
        
        match fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(cache) => {
                        debug!("Loaded controller cache from: {}", path.display());
                        cache
                    }
                    Err(e) => {
                        warn!("Failed to parse cache file: {}", e);
                        Self::new()
                    }
                }
            }
            Err(_) => {
                debug!("No existing cache file found at: {}", path.display());
                Self::new()
            }
        }
    }
    
    /// Save cache to disk
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = get_cache_path();
        
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        
        info!("Saved controller cache to: {}", path.display());
        Ok(())
    }
    
    /// Add or update a controller in the cache
    pub fn add_controller(&mut self, mac_address: String, side: Side, name: Option<String>) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let cached = CachedController {
            mac_address: mac_address.clone(),
            side: side.into(),
            name,
            last_seen: timestamp,
        };
        
        info!("Caching controller: {} ({:?})", mac_address, side);
        self.controllers.insert(mac_address, cached);
    }
    
    /// Get a controller from the cache by MAC address
    pub fn get_controller(&self, mac_address: &str) -> Option<&CachedController> {
        self.controllers.get(mac_address)
    }
    
    /// Get all controllers of a specific side
    pub fn get_by_side(&self, side: Side) -> Vec<&CachedController> {
        let cached_side: CachedSide = side.into();
        self.controllers
            .values()
            .filter(|c| c.side == cached_side)
            .collect()
    }
    
    /// Remove a controller from the cache
    pub fn remove_controller(&mut self, mac_address: &str) -> Option<CachedController> {
        self.controllers.remove(mac_address)
    }
    
    /// Clear all cached controllers
    pub fn clear(&mut self) {
        self.controllers.clear();
    }
    
    /// Get the number of cached controllers
    pub fn len(&self) -> usize {
        self.controllers.len()
    }
    
    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.controllers.is_empty()
    }
    
    /// List all cached controllers
    pub fn list_all(&self) -> Vec<&CachedController> {
        self.controllers.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_basic_operations() {
        let mut cache = ControllerCache::new();
        assert!(cache.is_empty());
        
        cache.add_controller("AA:BB:CC:DD:EE:FF".to_string(), Side::Left, Some("Left JoyCon".to_string()));
        assert_eq!(cache.len(), 1);
        
        let controller = cache.get_controller("AA:BB:CC:DD:EE:FF");
        assert!(controller.is_some());
        assert_eq!(controller.unwrap().side, CachedSide::Left);
    }
    
    #[test]
    fn test_cache_side_filtering() {
        let mut cache = ControllerCache::new();
        
        cache.add_controller("AA:BB:CC:DD:EE:01".to_string(), Side::Left, None);
        cache.add_controller("AA:BB:CC:DD:EE:02".to_string(), Side::Right, None);
        cache.add_controller("AA:BB:CC:DD:EE:03".to_string(), Side::Left, None);
        
        let left_controllers = cache.get_by_side(Side::Left);
        assert_eq!(left_controllers.len(), 2);
        
        let right_controllers = cache.get_by_side(Side::Right);
        assert_eq!(right_controllers.len(), 1);
    }
}
