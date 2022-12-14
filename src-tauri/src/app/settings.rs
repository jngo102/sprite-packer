use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Settings {
    #[serde(rename = "Language")]
    pub language: String,
    #[serde(rename = "Sprites Path")]
    pub sprites_path: String,
    #[serde(rename = "Theme")]
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            sprites_path: String::new(),
            theme: "Dark".to_string(),
        }
    }
}