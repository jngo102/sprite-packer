use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Settings {
    #[serde(rename = "Language")]
    pub language: String,
    #[serde(rename = "Sprites Path")]
    pub sprites_path: String,
    #[serde(rename = "Mode")]
    pub mode: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            sprites_path: String::new(),
            mode: "dark".to_string(),
        }
    }
}