use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct SpriteImage {
    pub sprite: Sprite, 
    pub image: image::DynamicImage,
}