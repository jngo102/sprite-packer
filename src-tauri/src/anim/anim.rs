use serde::{Deserialize, Serialize};

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
pub struct Animation {
    #[serde(rename="currentFrameIndex")]
    pub current_frame_index: u32,
    #[serde(rename="currentTime")]
    pub current_time: f32,
    pub duration: f32,
    pub fps: f32,
    pub frames: Vec<String>,
    #[serde(rename="loopStart")]
    pub loop_start: u32,
    pub name: String,
    #[serde(rename="numFrames")]
    pub num_frames: u32,
}

impl Animation {
    pub fn new(
        name: String,
        frames: Vec<String>,
        fps: f32,
        loop_start: u32,
    ) -> Self {
        let num_frames = frames.len() as u32;
        Self {
            current_frame_index: 0,
            current_time: 0.0,
            duration: (num_frames as f32) * (1.0 / fps),
            fps,
            frames,
            loop_start,
            name,
            num_frames
        }
    }
}