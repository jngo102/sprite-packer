use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    pub id: u32,
    pub x: i32,
    pub y: u32,
    pub xr: u32,
    pub yr: u32,
    pub width: u32,
    pub height: u32,
    pub collection_name: String,
    pub name: String,
    pub path: String,
    pub flipped: bool,
}