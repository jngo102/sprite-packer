use crate::tk2d::sprite::Sprite;
use serde::{de::{self, Visitor}, Deserialize, Deserializer, Serialize};
use std::cmp;
use std::fmt;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimInfo {
    pub fps: f32,
    #[serde(rename = "loopStart")]
    pub loop_start: u32,
    #[serde(rename = "numFrames")]
    pub num_frames: u32,
    #[serde(rename = "collectionName")]
    pub collection_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpriteInfo {
    #[serde(rename = "sid")]
    pub id: Vec<u32>,
    #[serde(rename = "sx")]
    pub x: Vec<i32>,
    #[serde(rename = "sy")]
    pub y: Vec<i32>,
    #[serde(rename = "sxr")]
    pub xr: Vec<i32>,
    #[serde(rename = "syr")]
    pub yr: Vec<i32>,
    #[serde(rename = "swidth")]
    pub width: Vec<i32>,
    #[serde(rename = "sheight")]
    pub height: Vec<i32>,
    #[serde(rename = "scollectionname")]
    pub collection_name: Vec<String>,
    #[serde(rename = "spath")]
    pub path: Vec<String>,
    #[serde(rename = "sfilpped")]
    pub flipped: Vec<bool>,
}

impl SpriteInfo {
    pub fn at(&self, index: usize) -> Option<Sprite> {
        let sprite_name = PathBuf::from(self.path[index].clone()).file_name()?.to_str()?.to_string();
        Some(Sprite {
            id: self.id[index],
            x: self.x[index],
            y: self.y[index],
            xr: self.xr[index],
            yr: self.yr[index],
            width: self.width[index],
            height: self.height[index],
            collection_name: self.collection_name[index].clone(),
            name: sprite_name,
            path: self.path[index].clone(),
            flipped: self.flipped[index],
        })
    }
}