// file_format.rs
//
// Serde structs for deserializing rooms.json
// Keeps JSON shape separate from runtime model

use serde::Deserialize;
use std::collections::HashMap;

/// Root structure of rooms.json
#[derive(Debug, Deserialize)]
pub struct RoomsFile {
    pub meta: FileMeta,
    pub start: FileCoord,
    pub layers: Vec<FileLayer>,
    pub rooms: Vec<FileRoom>,
    #[serde(default)]
    pub anchors: Vec<FileAnchor>,
}

#[derive(Debug, Deserialize)]
pub struct FileMeta {
    pub world_name: String,
    pub version: i32,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileCoord {
    pub layer: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Deserialize)]
pub struct FileLayer {
    pub layer: i32,
    pub name: String,
    pub scale: i32,
}

#[derive(Debug, Deserialize)]
pub struct FileRoom {
    pub id: String,
    pub pos: FileCoord,
    pub name: String,
    pub desc: String,
    #[serde(default)]
    pub exits: HashMap<String, FileExitSpec>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Exit destination supports either "to_rel": {dx,dy} OR "to_abs": {w,x,y}
#[derive(Debug, Deserialize)]
pub struct FileExitSpec {
    #[serde(default)]
    pub to_rel: Option<FileRel>,
    #[serde(default)]
    pub to_abs: Option<FileCoord>,
    #[serde(default)]
    pub desc: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileRel {
    pub dx: i32,
    pub dy: i32,
    #[serde(default)]
    pub dz: i32,
}

#[derive(Debug, Deserialize)]
pub struct FileAnchor {
    pub id: String,
    pub label: String,
    pub pos: FileCoord,
}
