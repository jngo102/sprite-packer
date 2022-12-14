use crate::app::settings::Settings;
use crate::tk2d::cln::Collection;
use crate::tk2d::anim::Animation;

pub struct App {
    pub loaded_collections: Vec<Collection>,
    pub loaded_animations: Vec<Animation>,
    pub run_task: bool,
    pub settings: Settings,
}

impl Default for App {
    fn default() -> Self {
        App {
            loaded_collections: Vec::new(),
            loaded_animations: Vec::new(),
            run_task: false,
            settings: Settings::default(),
        }
    }
}