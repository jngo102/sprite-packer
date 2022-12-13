use crate::tk2d::clip::Clip;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Animation {
    pub clips: Vec<Clip>,
    pub name: String,
}

impl Eq for Animation {
    fn assert_receiver_is_total_eq(&self) {
        self.name.assert_receiver_is_total_eq();
    }
}

impl Ord for Animation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for Animation {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for Animation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}