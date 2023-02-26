use std::{ops::ControlFlow, sync::Mutex};

use image::{DynamicImage, GenericImageView, SubImage};
use log::info;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub xr: i32,
    pub yr: i32,
    pub width: i32,
    pub height: i32,
    pub collection_name: String,
    pub name: String,
    pub path: String,
    pub flipped: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpriteImage {
    pub sprite: Sprite, 
    pub image: DynamicImage,
}

impl SpriteImage {
    pub fn equals(&self, other: &Self) -> bool {
        if self.sprite.collection_name != other.sprite.collection_name {
            info!("Collection names don't match: {} vs {}", self.sprite.collection_name, other.sprite.collection_name);
            return false;
        }
        let frame1 = self.trim();
        let frame2 = other.trim();
        if frame1.width() != frame2.width() || frame1.height() != frame2.height() {
            info!("Dimensions don't match: {}x{} vs {}x{}", frame1.width(), frame1.height(), frame2.width(), frame2.height());
            return false;
        }
        let result = Mutex::new(true);
        (0..frame1.width()).into_par_iter().try_for_each(|i| {
            (0..frame2.height()).into_par_iter().try_for_each(|j| {
                if frame1.get_pixel(i, j) != frame2.get_pixel(i, j) {
                    *result.lock().expect("Failed to lock result.") = false;
                    return ControlFlow::Break(());
                }
                return ControlFlow::Continue(());
            });
            if !*result.lock().expect("Failed to lock result.") {
                return ControlFlow::Break(());
            }
            return ControlFlow::Continue(());
        });

        return *result.lock().expect("Failed to lock result.");
    }

    pub fn trim(&self) -> SubImage<&DynamicImage> {
        let x_min = self.sprite.xr as u32;
        let y_min = (self.image.height() as i32 - self.sprite.yr - self.sprite.height) as u32;
        return self.image.view(x_min, y_min, self.sprite.width as u32, self.sprite.height as u32);
    }
}