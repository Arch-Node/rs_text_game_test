// multiplayer/types.rs
//
// STATUS: LEGACY - These types mirror SpacetimeDB tables but are not the source of truth.
// The actual backend uses SpacetimeDB tables defined in text_game_stdb/src/lib.rs
// and auto-generated SDK types in src/spacetimedb_client/*_type.rs
//
// POTENTIAL USES:
// - Bridge layer for client-side logic
// - Local caching/representation of SpacetimeDB data
// - May be replaced by SDK types as integration completes
//
// RUST LEARNING: Type Aliases and Newtypes
// - Type aliases (`type X = Y`) create synonyms for existing types
// - Newtypes (tuple structs) provide type safety and prevent mixing up similar types
// - `#[derive(...)]` automatically implements common traits
// - `Copy` allows bitwise copying, `Clone` allows explicit cloning
// - `PartialEq, Eq` enable equality comparisons
// - `Hash` allows use in HashMap/HashSet keys
// - `Debug` enables {:?} formatting for debugging

use std::fmt;

/// RUST LEARNING: Newtype Pattern
/// A newtype wraps a primitive type to provide type safety.
/// We can't accidentally use a SessionId where a PlayerId is expected!
/// 
/// `#[derive(Copy)]` means the type can be copied with a simple bit copy (cheap).
/// This only works for types that don't own heap data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccountId(pub u64);

// RUST LEARNING: Implementing Display
// The Display trait controls how a type is formatted with `{}` (not `{:?}`)
// This is for user-facing output, while Debug is for developer output
impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Player#{}", self.0)
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Session#{}", self.0)
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Account#{}", self.0)
    }
}

// TODO: Add methods for generating unique IDs
// Consider using atomic counters or UUIDs
impl PlayerId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl SessionId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl AccountId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Connection identifiers for different interfaces
/// 
/// RUST LEARNING: Struct with Option<String> fields
/// - Option<T> represents "maybe has a value"
/// - String is owned, heap-allocated
/// - Clone is needed to duplicate this data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionIdentifiers {
    /// Signal messenger ID (username or phone in Signal format)
    pub signal_id: Option<String>,
    
    /// Phone number (E.164 format recommended: +1234567890)
    pub phone_number: Option<String>,
    
    /// IP address (for terminal connections)
    pub ip_address: Option<String>,
    
    /// MAC address (for device identification)
    pub mac_address: Option<String>,
    
    /// Device fingerprint (browser, terminal client, etc.)
    pub device_fingerprint: Option<String>,
}

impl Default for ConnectionIdentifiers {
    fn default() -> Self {
        Self {
            signal_id: None,
            phone_number: None,
            ip_address: None,
            mac_address: None,
            device_fingerprint: None,
        }
    }
}

impl ConnectionIdentifiers {
    /// Create identifiers for Signal interface
    pub fn from_signal(signal_id: String) -> Self {
        Self {
            signal_id: Some(signal_id),
            ..Default::default()
        }
    }
    
    /// Create identifiers for SMS interface
    pub fn from_sms(phone_number: String) -> Self {
        Self {
            phone_number: Some(phone_number),
            ..Default::default()
        }
    }
    
    /// Create identifiers for Terminal interface
    pub fn from_terminal(ip_address: String, mac_address: Option<String>) -> Self {
        Self {
            ip_address: Some(ip_address),
            mac_address,
            ..Default::default()
        }
    }
    
    /// Get primary identifier for display
    pub fn primary_id(&self) -> String {
        self.signal_id
            .as_ref()
            .or(self.phone_number.as_ref())
            .or(self.ip_address.as_ref())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string())
    }
}
