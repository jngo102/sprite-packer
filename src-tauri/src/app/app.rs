use crate::app::settings::Settings;
use crate::tk2d::cln::Collection;
use crate::tk2d::anim::Animation;

#[derive(Clone)]
pub struct App {
    pub current_pack_progress: usize,
    pub current_sprite_num: usize,
    pub loaded_collections: Vec<Collection>,
    pub loaded_animations: Vec<Animation>,
    pub settings: Settings,
}

impl Default for App {
    fn default() -> Self {
        App {
            current_pack_progress: 0,
            current_sprite_num: 0,
            loaded_collections: Vec::new(),
            loaded_animations: Vec::new(),
            settings: Settings::default(),
        }
    }
}