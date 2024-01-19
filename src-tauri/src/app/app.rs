use crate::app::settings::Settings;
use crate::tk2d::anim::Animation;
use crate::tk2d::cln::Collection;

pub struct App {
    pub loaded_collections: Vec<Collection>,
    pub loaded_animations: Vec<Animation>,
    pub settings: Settings,
}

impl Default for App {
    fn default() -> Self {
        App {
            loaded_collections: Vec::new(),
            loaded_animations: Vec::new(),
            settings: Settings::default(),
        }
    }
}