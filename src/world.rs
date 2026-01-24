// world.rs
//
// World loading and validation

use std::collections::HashMap;
use std::fs;

use crate::models::{World, Meta, Layer, Room, ExitSpec, ExitTo, Anchor, Coord};
use crate::file_format::RoomsFile;
use crate::exits::{exit_kind, resolve_exit_target};

/// Load and validate world from a rooms.json file
pub fn load_world_from_rooms_json(path: &str) -> Result<World, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("Failed to read {path}: {e}"))?;
    let file: RoomsFile =
        serde_json::from_str(&raw).map_err(|e| format!("Failed to parse JSON in {path}: {e}"))?;

    // Convert meta
    let meta = Meta {
        world_name: file.meta.world_name,
        version: file.meta.version,
        notes: file.meta.notes,
    };

    // Layers -> map
    let mut layers = HashMap::new();
    for l in file.layers {
        if l.scale <= 0 {
            return Err(format!("Layer {} has invalid scale {} (must be > 0)", l.layer, l.scale));
        }
        layers.insert(
            l.layer,
            Layer {
                layer: l.layer,
                name: l.name,
                scale: l.scale,
            },
        );
    }

    // Start
    let start = Coord {
        layer: file.start.layer,
        x: file.start.x,
        y: file.start.y,
        z: file.start.z,
    };

    // Rooms -> coord map + basic exit conversion
    let mut rooms_by_coord: HashMap<Coord, Room> = HashMap::new();
    for r in file.rooms {
        let pos = Coord {
            layer: r.pos.layer,
            x: r.pos.x,
            y: r.pos.y,
            z: r.pos.z,
        };

        if rooms_by_coord.contains_key(&pos) {
            return Err(format!(
                "Duplicate room coordinate at layer={}, x={}, y={}, z={}",
                pos.layer, pos.x, pos.y, pos.z
            ));
        }

        let mut exits: HashMap<String, ExitSpec> = HashMap::new();
        for (exit_id, spec) in r.exits {
            let to = match (spec.to_rel, spec.to_abs) {
                (Some(rel), None) => ExitTo::Relative { dx: rel.dx, dy: rel.dy, dz: rel.dz },
                (None, Some(abs)) => ExitTo::Absolute(Coord { layer: abs.layer, x: abs.x, y: abs.y, z: abs.z }),
                (None, None) => return Err(format!("Room '{}' exit '{}' missing to_rel/to_abs", r.id, exit_id)),
                (Some(_), Some(_)) => return Err(format!("Room '{}' exit '{}' has BOTH to_rel and to_abs", r.id, exit_id)),
            };

            exits.insert(
                exit_id,
                ExitSpec {
                    to,
                    desc: spec.desc,
                },
            );
        }

        rooms_by_coord.insert(
            pos,
            Room {
                id: r.id,
                pos,
                name: r.name,
                desc: r.desc,
                exits,
                tags: r.tags,
            },
        );
    }

    // Anchors -> id map
    let mut anchors_by_id: HashMap<String, Anchor> = HashMap::new();
    for a in file.anchors {
        if anchors_by_id.contains_key(&a.id) {
            return Err(format!("Duplicate anchor id '{}'", a.id));
        }
        anchors_by_id.insert(
            a.id.clone(),
            Anchor {
                id: a.id,
                label: a.label,
                pos: Coord { layer: a.pos.layer, x: a.pos.x, y: a.pos.y, z: a.pos.z },
            },
        );
    }

    // Validation pass
    validate_world(&layers, &rooms_by_coord, &anchors_by_id, start)?;

    Ok(World {
        meta,
        layers,
        rooms_by_coord,
        anchors_by_id,
        start,
    })
}

/// Validate world consistency
fn validate_world(
    layers: &HashMap<i32, Layer>,
    rooms_by_coord: &HashMap<Coord, Room>,
    anchors_by_id: &HashMap<String, Anchor>,
    start: Coord,
) -> Result<(), String> {
    if !rooms_by_coord.contains_key(&start) {
        return Err(format!(
            "Start coordinate does not have a room: layer={}, x={}, y={}, z={}",
            start.layer, start.x, start.y, start.z
        ));
    }

    for (layer, _layer_info) in layers {
        // optional: enforce that layer has at least one room
        let has_any = rooms_by_coord.keys().any(|c| &c.layer == layer);
        if !has_any {
            // not fatal, but you might want it to be later
            // return Err(format!("Layer {} has no rooms", layer));
        }
    }

    for a in anchors_by_id.values() {
        if !rooms_by_coord.contains_key(&a.pos) {
            return Err(format!(
                "Anchor '{}' points to missing room at layer={}, x={}, y={}, z={}",
                a.id, a.pos.layer, a.pos.x, a.pos.y, a.pos.z
            ));
        }
    }

    // Validate exits
    for room in rooms_by_coord.values() {
        for (exit_id, spec) in &room.exits {
            // Optional: warn/validate exit_id is known in engine defs
            // (We allow unknown exit IDs; they become Special by default)
            let _kind = exit_kind(exit_id);

            let target = resolve_exit_target(room.pos, spec)?;
            if !rooms_by_coord.contains_key(&target) {
                return Err(format!(
                    "Room '{}' exit '{}' points to missing room at layer={}, x={}, y={}, z={}",
                    room.id, exit_id, target.layer, target.x, target.y, target.z
                ));
            }
        }
    }

    Ok(())
}
