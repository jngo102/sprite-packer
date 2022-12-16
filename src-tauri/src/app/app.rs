use crate::app::settings::Settings;
use crate::tk2d::anim::Animation;
use crate::tk2d::cln::Collection;
use crate::tk2d::sprite::Sprite;

pub struct App {
    pub changed_sprites: Vec<Sprite>,
    pub loaded_collections: Vec<Collection>,
    pub loaded_animations: Vec<Animation>,
    pub settings: Settings,
}

impl Default for App {
    fn default() -> Self {
        App {
            changed_sprites: Vec::new(),
            loaded_collections: Vec::new(),
            loaded_animations: Vec::new(),
            settings: Settings::default(),
        }
    }
}