use crate::app::settings::Settings;
use crate::tk2d::cln::Collection;
use crate::tk2d::lib::Library;

#[derive(Clone)]
pub struct App {
    pub loaded_collections: Vec<Collection>,
    pub loaded_libraries: Vec<Library>,
    pub current_pack_progress: usize,
    pub settings: Settings,
}

impl Default for App {
    fn default() -> Self {
        App {
            loaded_collections: Vec::new(),
            loaded_libraries: Vec::new(),
            current_pack_progress: 0,
            settings: Settings::default(),
        }
    }
}