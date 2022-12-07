use crate::app::settings::Settings;

pub struct App {
    pub current_pack_progress: u8,
    pub current_frame: String,
    pub settings: Settings,
}

impl Default for App {
    fn default() -> Self {
        App {
            current_pack_progress: 0,
            current_frame: String::new(),
            settings: Settings::default(),
        }
    }
}