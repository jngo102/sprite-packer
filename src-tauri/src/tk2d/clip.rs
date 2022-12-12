use serde::{Deserialize, Serialize};
use crate::tk2d::sprite::Sprite;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Clip {
    #[serde(rename="currentFrameIndex")]
    pub current_frame_index: u32,
    #[serde(skip_deserializing)]
    #[serde(rename = "currentTime")]
    pub current_time: f32,
    #[serde(skip_deserializing)]
    pub duration: f32,
    pub fps: f32,
    #[serde(skip_deserializing)]
    pub frames: Vec<Sprite>,
    #[serde(skip_deserializing)]
    pub frame_names: Vec<String>,
    #[serde(rename="loopStart")]
    pub loop_start: u32,
    #[serde(skip_deserializing)]
    pub name: String,
    #[serde(rename="numFrames")]
    pub num_frames: u32,
}

impl Clip {
    pub fn new(
        name: String,
        frames: Vec<Sprite>,
        fps: f32,
        loop_start: u32,
    ) -> Self {
        let num_frames = frames.len();
        Self {
            current_frame_index: 0,
            current_time: 0.0,
            duration: (num_frames as f32) * (1.0 / fps),
            fps,
            frames: frames.clone(),
            frame_names: frames.iter().map(|frame| frame.name.clone()).collect(),
            loop_start,
            name,
            num_frames: num_frames as u32
        }
    }
}

impl Eq for Clip {
    fn assert_receiver_is_total_eq(&self) {
        self.name.assert_receiver_is_total_eq();
    }
}

impl Ord for Clip {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for Clip {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for Clip {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}