// exits.rs
//
// Exit definitions, aliases, and exit-related helper functions

use std::collections::HashMap;
use crate::models::{Coord, ExitSpec, ExitTo};

/// Types of exits in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitKind {
    /// Moves by (dx,dy) within same layer.
    Directional,
    /// Teleport to a target (absolute) within same layer for Teleporter class.
    Teleport,
    /// Layer shift (not implemented as an "exit" yet, but reserved).
    LayerShift,
    /// Generic special exit (window, mousehole) that can still be Relative/Absolute.
    Special,
}

/// Exit definition with aliases
#[derive(Debug)]
pub struct ExitDef {
    pub id: &'static str,
    pub kind: ExitKind,
    pub aliases: &'static [&'static str],
}

/// Available exit definitions
/// You can expand this list as needed
pub const EXIT_DEFS: &[ExitDef] = &[
    ExitDef { id: "north", kind: ExitKind::Directional, aliases: &["n", "north"] },
    ExitDef { id: "south", kind: ExitKind::Directional, aliases: &["s", "south"] },
    ExitDef { id: "east",  kind: ExitKind::Directional, aliases: &["e", "east"] },
    ExitDef { id: "west",  kind: ExitKind::Directional, aliases: &["w", "west"] },
    ExitDef { id: "up",    kind: ExitKind::Directional, aliases: &["u", "up"] },
    ExitDef { id: "down",  kind: ExitKind::Directional, aliases: &["d", "down"] },
    // Examples of special exits; rooms must explicitly include them.
    ExitDef { id: "window", kind: ExitKind::Special, aliases: &["window", "climb window"] },
    ExitDef { id: "mousehole", kind: ExitKind::Special, aliases: &["mousehole", "crawl", "crawl mousehole"] },
];

/// Build an alias -> exit_id lookup map
pub fn build_exit_alias_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for def in EXIT_DEFS {
        // include id itself as an alias
        map.insert(def.id.to_string(), def.id.to_string());
        for &a in def.aliases {
            map.insert(a.to_string(), def.id.to_string());
        }
    }
    map
}

/// Get the kind of an exit by its ID
pub fn exit_kind(exit_id: &str) -> ExitKind {
    EXIT_DEFS
        .iter()
        .find(|d| d.id == exit_id)
        .map(|d| d.kind)
        .unwrap_or(ExitKind::Special)
}

/// Resolve the target coordinate for an exit
pub fn resolve_exit_target(from: Coord, spec: &ExitSpec) -> Result<Coord, String> {
    match spec.to {
        ExitTo::Relative { dx, dy, dz } => Ok(Coord { 
            layer: from.layer, 
            x: from.x + dx, 
            y: from.y + dy, 
            z: from.z + dz 
        }),
        ExitTo::Absolute(c) => Ok(c),
    }
}
