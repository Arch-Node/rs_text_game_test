// models.rs
//
// Core data structures for the text adventure game engine

use std::collections::HashMap;

/// Coordinate in 3D space with layer (layer = which dimension/realm, x,y,z = position within)
/// 
/// **Note:** This legacy single-player code uses `layer` to represent what the multiplayer
/// SpacetimeDB backend calls `dimension` (material, ethereal, shadow, dream). Both refer to
/// the same concept: separate 3D spatial planes that can shift/interact but maintain
/// independent coordinate systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord {
    pub layer: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Player state
#[derive(Debug, Clone)]
pub struct Player {
    pub class: PlayerClass,
    pub pos: Coord,
}

/// Player class determines available abilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerClass {
    Teleporter,
    LayerWalker,
}

impl PlayerClass {
    pub fn can_teleport(self) -> bool {
        matches!(self, PlayerClass::Teleporter)
    }
    pub fn can_shift_layers(self) -> bool {
        matches!(self, PlayerClass::LayerWalker)
    }
}

/// Complete game state
#[derive(Debug)]
pub struct GameState {
    pub world: World,
    pub player: Player,
    pub is_running: bool,
}

/// The validated, ready-to-use runtime world container
#[derive(Debug)]
pub struct World {
    pub meta: Meta,
    pub layers: HashMap<i32, Layer>,
    pub rooms_by_coord: HashMap<Coord, Room>,
    pub anchors_by_id: HashMap<String, Anchor>,
    pub start: Coord,
}

/// Room in the game world
#[derive(Debug, Clone)]
pub struct Room {
    pub id: String,
    pub pos: Coord,
    pub name: String,
    pub desc: String,
    /// Exits are explicit: if an exit isn't listed here, it doesn't exist.
    /// Key is exit_id like "north", "window", "mousehole", "rift".
    pub exits: HashMap<String, ExitSpec>,
    pub tags: Vec<String>,
}

/// Exit specification
#[derive(Debug, Clone)]
pub struct ExitSpec {
    pub to: ExitTo,
    pub desc: Option<String>,
}

/// Exit destination (either relative or absolute coordinates)
#[derive(Debug, Clone)]
pub enum ExitTo {
    Relative { dx: i32, dy: i32, dz: i32 },
    Absolute(Coord),
}

/// Teleport anchor point
#[derive(Debug, Clone)]
pub struct Anchor {
    pub id: String,
    pub label: String,
    pub pos: Coord,
}

/// Layer definition
#[derive(Debug, Clone)]
pub struct Layer {
    pub layer: i32,
    pub name: String,
    pub scale: i32, // scale relative to base (layer 0)
}

/// World metadata
#[derive(Debug, Clone)]
pub struct Meta {
    pub world_name: String,
    pub version: i32,
    pub notes: Option<String>,
}
