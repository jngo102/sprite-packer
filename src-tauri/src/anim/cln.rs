use crate::anim::anim::Animation;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collection {
    pub animations: Vec<Animation>,
    pub name: String,
}