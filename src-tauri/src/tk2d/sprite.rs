use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sprite {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub xr: i32,
    pub yr: i32,
    pub width: i32,
    pub height: i32,
    pub collection_name: String,
    pub name: String,
    pub path: String,
    pub flipped: bool,
}