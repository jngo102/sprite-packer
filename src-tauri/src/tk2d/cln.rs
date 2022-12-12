use crate::tk2d::sprite::Sprite;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Collection {
    pub name: String,
    pub path: PathBuf,
    pub sprites: Vec<Sprite>,
}

impl Eq for Collection {
    fn assert_receiver_is_total_eq(&self) {
        self.name.assert_receiver_is_total_eq();
    }
}

impl Ord for Collection {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for Collection {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for Collection {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}