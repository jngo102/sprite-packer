#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod macros;
mod tk2d;
mod util;

use app::app::App;
use app::settings::Settings;
use notify::event::ModifyKind;
use tk2d::anim::*;
use tk2d::clip::Clip;
use tk2d::cln::Collection;
use tk2d::info::{AnimInfo, SpriteInfo};
use tk2d::sprite::Sprite;
use util::fs as fs_util;
use image::{DynamicImage, GenericImage, GenericImageView};
use log::{error, info, LevelFilter, warn};
use notify::{EventKind, RecursiveMode, Watcher};
use rayon::prelude::*;
use serde::Serialize;
use simple_logging;
use std::cmp;
use std::env;
use std::fs;
use std::ops::ControlFlow;
use std::path::{PathBuf, Path};
use std::str::FromStr;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, self, Sender};
use std::time::Instant;
use tauri::{AppHandle, command, CustomMenuItem, Manager, Menu, MenuItem, State, Submenu, Window};
use tauri::api::dialog::blocking::FileDialogBuilder;
use tauri::async_runtime;
use tauri::RunEvent::ExitRequested;

/// A data structure containing the current pack progress
#[derive(Clone, Serialize)]
struct ProgressPayload {
    progress: usize
}

struct AppState(Mutex<App>);

static mut CHANGED_SPRITES: Mutex<Vec<Sprite>> = Mutex::new(Vec::new());
static mut TX_PROGRESS: Mutex<Option<Sender<()>>> = Mutex::new(None);
static mut RX_PROGRESS: Mutex<Option<Receiver<()>>> = Mutex::new(None);

/// The name of the folder containing the log and settings files
const APP_NAME: &str = "sprite-packer";

/// Backup existing sprites into app data folder.
/// # Arguments
/// * `state` - The application state
fn backup_sprites(state: &AppState) {
    let settings = state.0.lock().expect("Failed to lock settings.").settings.clone();
    let sprites_path = PathBuf::from(settings.sprites_path.clone());
            let backup_sprites_path = match confy::get_configuration_file_path(APP_NAME, APP_NAME) {
                Ok(settings_path) => {
                    match settings_path.parent() {
                        Some(settings_dir) => {
                            match settings_dir.parent() {
                                Some(app_dir) => app_dir.join("Backup"),
                                None => log_panic!("Settings directory does not have a parent."),
                            }
                        }
                        None => log_panic!("Failed to get parent of settings path: {}", settings_path.display())
                    }
                },
                Err(e) => log_panic!("Failed to get config file path: {}", e),
            };
            match fs_util::copy_dir_all(sprites_path, backup_sprites_path) {
                Ok(_) => info!("Backed up existing sprites."),
                Err(e) => log_panic!("Failed to backup sprites: {}", e),
            }
}

#[command]
fn check_for_changed_sprites(already_changed_sprites: Vec<Sprite>, state: State<AppState>) -> Vec<Sprite> {
    let app_state = state.0.lock().expect("Failed to lock app state.");
    unsafe {
        let sprites: &mut Vec<Sprite> = &mut CHANGED_SPRITES.lock().expect("Failed to lock CHANGED_SPRITES");
        let mut sprites: Vec<Sprite> = sprites.par_iter().map(|sprite| get_collection(sprite.collection_name.clone(), app_state.loaded_collections.clone()).sprites.par_iter().find_first(|s| s.id == sprite.id).unwrap().clone()).collect();
        sprites.retain(|sprite| !already_changed_sprites.contains(sprite));
        return sprites;
    }
}

/// Replace all duplicate sprites in a collection
/// # Arguments
/// * `source_sprite` - The sprite to replace duplicates with
/// * `state` - The application state
/// # Returns
/// * `Vec<Sprite>` A list of all duplicate sprites in the collection
#[command]
fn replace_duplicate_sprites(source_sprite: Sprite, state: State<AppState>) {
    let sprites_path: PathBuf;
    {
        let app_state = state.0.lock().expect("Failed to lock app state.");
        sprites_path = PathBuf::from(app_state.settings.sprites_path.clone());
    }

    let source_path = if sprites_path.join(source_sprite.path.clone()).exists() {
        sprites_path.join(source_sprite.path.clone())
    } else if PathBuf::from(source_sprite.path.clone()).exists() {
        PathBuf::from(source_sprite.path.clone())
    } else {
        log_panic!("Failed to get a valid path from source sprite at {}", source_sprite.path);
    };
    match fs::read_dir(sprites_path.clone()) {
        Ok(anim_paths) => {
            for anim_path in anim_paths {
                match anim_path {
                    Ok(anim_path) => {
                        match fs::read_dir(anim_path.path()) {
                            Ok(clip_paths) => {
                                for path in clip_paths {
                                    match path {
                                        Ok(clip_path) => {
                                            if clip_path.file_name() == "0.Atlases" {
                                                continue;
                                            }

                                            match fs::read_dir(clip_path.path()) {
                                                Ok(sprite_paths) => {
                                                    for path in sprite_paths {
                                                        match path {
                                                            Ok(sprite_path) => {
                                                                if sprite_path.path().extension().unwrap() != "png" {
                                                                    continue;
                                                                }

                                                                let sprite_name = sprite_path.file_name().into_string().expect("Failed to convert sprite name to string.");
                                                                let sprite_data = sprite_name.split("-").collect::<Vec<&str>>();
                                                                let frame_id = sprite_data[sprite_data.len() - 1].replace(".png", "");
                                                                if sprite_name != source_sprite.name && 
                                                                    frame_id.parse::<u32>().unwrap() == source_sprite.id && 
                                                                    get_collection_from_sprite_name(sprite_name.clone(), state.clone()).name == source_sprite.collection_name {
                                                                    match fs::copy(source_path.clone(), sprite_path.path()) {
                                                                        Ok(_) => info!("Replaced duplicate sprite {} with {}", sprite_name, source_sprite.name),
                                                                        Err(e) => log_panic!("Failed to replace duplicate sprite: {}", e)
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => log_panic!("Failed to read sprite path: {}", e)
                                                            }
                                                    }
                                                }
                                                Err(e) => log_panic!("Failed to read clip path: {}", e)
                                            }
                                        }
                                        Err(e) => log_panic!("Failed to read directory: {}", e)
                                    }
                                }
                            }
                            Err(e) => log_panic!("Failed to read directory: {}", e)
                        }
                    }
                    Err(e) => log_panic!("Error while iterating path: {}", e),
                }
            }
        }
        Err(e) => log_panic!("Failed to read directory: {}", e),
    }
}

/// Get a collection by its name
/// # Arguments
/// * `collection_name` - The name of the collection
/// * `collections` - A list of collections to search throuhg
/// # Returns
/// *`Collection` The collection with the given name
fn get_collection(collection_name: String, collections: Vec<Collection>) -> Collection {
    return match collections.par_iter().find_first(|cln| cln.name == collection_name) {
        Some(collection) => collection.clone(),
        None => log_panic!("Failed to find collection: {}", collection_name),
    };
}

fn main() {
    setup_app();
}

/// Set up the application
fn setup_app() {
    let app_state = AppState(Default::default());
    match confy::load::<Settings>(APP_NAME, APP_NAME) { 
        Ok(settings) => {
            app_state.0.lock().expect("Failed to lock app_state").settings = settings.clone();
            match confy::get_configuration_file_path(APP_NAME, APP_NAME) {
                Ok(settings_path) => {
                    match settings_path.parent() {
                        Some(settings_dir) => {
                            let log_path = settings_dir.join(format!("{}.log", APP_NAME));
                            match simple_logging::log_to_file(log_path.clone(), LevelFilter::Info) {
                                Ok(_) => info!("Opened logger at: {}", log_path.display()),
                                Err(e) => log_panic!("Failed to open logger: {}", e)
                            }
                            // match settings_dir.parent() {
                            //     Some(app_dir) => {
                            //         if app_dir.join("Backup").exists() {
                            //             load_backup_sprites(&app_state, app_dir.join("Backup"));
                            //         } else {
                            //             match fs::create_dir(app_dir.join("Backup")) {
                            //                 Ok(_) => info!("Created backup directory at: {}", app_dir.join("Backup").display()),
                            //                 Err(e) => log_panic!("Failed to create backup directory: {}", e),
                            //             }
                            //         }
                            //     },
                            //     None => panic!("Settings directory does not have a parent."),
                            // }
                        }
                        None => log_panic!("Failed to get parent of settings path: {}", settings_path.display())
                    }
                }
                Err(e) => log_panic!("Failed to get settings path: {}", e)
            }
            if settings.sprites_path == "".to_string() {
                select_sprites_path(&app_state);
            }
        },
        Err(e) => log_panic!("Failed to load settings: {}", e)
    }
    
    load_collections_and_animations(&app_state);

    let sprites_path = app_state.0.lock().expect("Failed to lock app_state").settings.sprites_path.clone();
    async_runtime::spawn(async move {
        start_watcher(sprites_path);
    });

    let backup = CustomMenuItem::new("backup_sprites", "Backup Sprites");
    let refresh = CustomMenuItem::new("refresh", "Refresh").accelerator("F5");
    let set_sprites_path = CustomMenuItem::new("set_sprites_path", "Set Sprites Path");
    let quit = CustomMenuItem::new("quit", "Quit").accelerator("Alt+F4");
    let submenu = Menu::new()
        .add_item(backup)
        .add_item(refresh)
        .add_item(set_sprites_path)
        .add_native_item(MenuItem::Separator)
        .add_item(quit);
    let options_menu = Submenu::new("Options", submenu);
    let menu = Menu::new().add_submenu(options_menu);

    let app = tauri::Builder::default()
        .manage(app_state)
        .menu(menu)
        .on_menu_event(|event| match event.menu_item_id() {
            "backup_sprites" => {
                let app_handle = event.window().app_handle();
                let state = app_handle.state::<AppState>();
                backup_sprites(&state);
            }
            "quit" => event.window().close().expect("Failed to close window from Options menu"),
            "refresh" => event.window().emit("refresh", ()).expect("Failed to emit refresh event"),
            "set_sprites_path" => {
                let app_handle = event.window().app_handle();
                let state = app_handle.state::<AppState>();
                select_sprites_path(&state);
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            cancel_pack,
            check_for_changed_sprites,
            debug,
            get_animation,
            get_animation_name_from_collection_name,
            get_animation_list,
            get_collection_from_sprite_name,
            get_collections_from_animation_name,
            get_language,
            get_sprites_path,
            get_mode,
            pack_single_collection,
            replace_duplicate_sprites,
            set_language,
            set_mode
        ])
        .build(tauri::generate_context!())
        .expect("Failed to build tauri application.");

    app.run(move |app_handle, event| match event {
        ExitRequested { api, .. } => {
            api.prevent_exit(); 

            let state = app_handle.state::<AppState>();
            let settings = state.0.lock().expect("Failed to lock app_state").settings.clone();
            confy::store(APP_NAME, APP_NAME, settings).expect("Failed to save settings");

            app_handle.exit(0);
        }
        _ => {}
    });
}

/// Load backup sprites from app data folder.
/// # Arguments
/// * `state` - The application state
/// * `sprites_path` - The path to the backup sprites folder
fn load_backup_sprites(state: &AppState, sprites_path: PathBuf) {
    let mut app_state = state.0.lock().expect("Failed to lock app_state");
    match fs::read_dir(sprites_path.clone()) {
        Ok(anim_paths) => {
            for anim_path in anim_paths {
                match anim_path {
                    Ok(anim_path) => {
                        let sprite_info_path =
                            anim_path.path().join("0.Atlases").join("SpriteInfo.json");
                        match fs::read_to_string(sprite_info_path) {
                            Ok(text) => {
                                let sprite_info: SpriteInfo = match serde_json::from_str(&text) {
                                    Ok(info) => info,
                                    Err(e) => log_panic!("Failed to parse SpriteInfo.json: {}", e),
                                };

                                for i in 0..sprite_info.id.len() {
                                    match sprite_info.at(i) {
                                        Some(sprite) => {
                                            if sprites_path.join(sprite.path.clone()).exists() {
                                                let backup_sprite = image::open(sprites_path.join(sprite.path.clone())).expect("Failed to open backup sprite");
                                                app_state.backup_sprites.insert(sprite, backup_sprite);
                                            } else if PathBuf::from(sprite.clone().path.clone()).exists() {
                                                let backup_sprite = image::open(sprite.path.clone()).expect("Failed to open backup sprite");
                                                app_state.backup_sprites.insert(sprite, backup_sprite);
                                            }
                                        }
                                        None => log_panic!("Failed to get sprite at index {}.", i),
                                    }
                                }
                            }
                            Err(e) => log_panic!("Failed to read SpriteInfo.json: {}", e),
                        }
                    }
                    Err(e) => log_panic!("Error while iterating path: {}", e),
                }
            }
        }
        Err(e) => log_panic!("Failed to read directory: {}", e),
    }
}

/// Load collections and animations from sprite files on disk
/// # Arguments
/// * `state` - The application state
fn load_collections_and_animations(state: &AppState) {
    let mut app_state = state.0.lock().expect("Failed to lock app state");
    match fs::read_dir(app_state.settings.sprites_path.clone()) {
        Ok(anim_paths) => {
            for anim_path in anim_paths {
                match anim_path {
                    Ok(anim_path) => {
                        let sprite_info_path =
                            anim_path.path().join("0.Atlases").join("SpriteInfo.json");
                        match fs::read_to_string(sprite_info_path) {
                            Ok(text) => {
                                let sprite_info: SpriteInfo = match serde_json::from_str(&text) {
                                    Ok(info) => info,
                                    Err(e) => log_panic!("Failed to parse SpriteInfo.json: {}", e),
                                };

                                for i in 0..sprite_info.id.len() {
                                    match sprite_info.at(i) {
                                        Some(sprite) => {
                                            match app_state.loaded_collections.iter().find(|cln| cln.name == sprite.collection_name) {
                                                Some(collection) => {
                                                    let mut collection = collection.clone();
                                                    collection.sprites.push(sprite.clone());
                                                    app_state.loaded_collections.retain(|cln| cln.name != collection.name);
                                                    app_state.loaded_collections.push(collection);
                                                }
                                                None => {
                                                    let collection_name = sprite.clone().collection_name;
                                                    let mut cln = Collection {
                                                        name: collection_name.clone(),
                                                        path: anim_path.path().join("0.Atlases").join(format!("{}.png", collection_name)),
                                                        sprites: Vec::new(),
                                                    };
                                                    cln.sprites.push(sprite);
                                                    app_state.loaded_collections.push(cln);
                                                }
                                            }
                                        }
                                        None => log_panic!("Failed to get sprite at index {}.", i),
                                    }
                                }

                                let clips = Mutex::new(Vec::new());
                                match fs::read_dir(anim_path.path()) {
                                    Ok(clip_paths) => {
                                        clip_paths.into_iter().par_bridge().for_each(|clip_path| {
                                            match clip_path {
                                                Ok(clip_path) => {
                                                    match clip_path.path().file_name() {
                                                        Some(file_name) => {
                                                            if file_name == "0.Atlases" {
                                                                return;
                                                            }
                                                        },
                                                        None => log_panic!("Failed to get file name of clip path {:?}", clip_path.path().display()),
                                                    }
                                                    let frames = Mutex::new(Vec::new());
                                                    let fps = Mutex::new(12.0);
                                                    let loop_start = Mutex::new(0);
                                                    match fs::read_dir(clip_path.path()) {
                                                        Ok(frame_paths) => {
                                                            frame_paths.into_iter().par_bridge().for_each(|frame_path| {
                                                                match frame_path {
                                                                    Ok(frame_path) => {
                                                                        if frame_path.file_name() == "AnimInfo.json" {
                                                                            match fs::read_to_string(frame_path.path()) {
                                                                                Ok(text) => {
                                                                                    let anim_info: AnimInfo = match serde_json::from_str(&text) {
                                                                                        Ok(anim_info) => anim_info,
                                                                                        Err(e) => log_panic!("Failed to parse AnimInfo.json: {}", e),
                                                                                    };
                                                                                    match fps.lock() {
                                                                                        Ok(mut fps) => *fps = anim_info.fps,
                                                                                        Err(e) => log_panic!("Failed to lock fps: {}", e),
                                                                                    }
                                                                                    match loop_start.lock() {
                                                                                        Ok(mut loop_start) => *loop_start = anim_info.loop_start,
                                                                                        Err(e) => log_panic!("Failed to lock loop_start: {}", e),
                                                                                    }
                                                                                },
                                                                                Err(e) => log_panic!("Failed to read AnimInfo.json: {}", e),
                                                                            }
                                                                            return;
                                                                        }
                                                                        
                                                                        let sprite = match sprite_info.path.par_iter().position_first(|path| frame_path.path().ends_with(path)) {
                                                                            Some(index) => {
                                                                                match sprite_info.at(index) {
                                                                                    Some(sprite) => sprite,
                                                                                    None => log_panic!("Failed to get sprite at index {}", index),
                                                                                }
                                                                            },
                                                                            None => log_panic!("Failed to find sprite with path {:?}", frame_path.path().display()),
                                                                        };

                                                                        match frames.lock() {
                                                                            Ok(mut frames) => frames.push(sprite),
                                                                            Err(e) => log_panic!("Failed to lock frames: {}", e),
                                                                        }
                                                                    }
                                                                    Err(e) => log_panic!("Failed to get frame path: {}", e),
                                                                }
                                                            });
                                                        }
                                                        Err(e) => log_panic!(
                                                            "Failed to read directory {:?}: {}",
                                                            anim_path, e
                                                        ),
                                                    }

                                                    match clip_path.file_name().to_str() {
                                                        Some(clip_name) => {
                                                            let frames = match frames.lock() {
                                                                Ok(frames) => frames.clone(),
                                                                Err(e) => log_panic!("Failed to lock frames: {}", e),
                                                            };
                                                            let fps = match fps.lock() {
                                                                Ok(fps) => *fps,
                                                                Err(e) => log_panic!("Failed to lock fps: {}", e),
                                                            };
                                                            let loop_start = match loop_start.lock() {
                                                                Ok(loop_start) => *loop_start,
                                                                Err(e) => log_panic!("Failed to lock loop_start: {}", e),
                                                            };
                                                            match clips.lock() {
                                                                Ok(mut clips) => clips.push(Clip::new(
                                                                    clip_name.to_string(),
                                                                    frames,
                                                                    fps,
                                                                    loop_start,
                                                                )),
                                                                Err(e) => log_panic!("Failed to lock clips: {}", e),
                                                            }
                                                        }
                                                        None => log_panic!("Failed to get clip name."),
                                                    }
                                                }
                                                Err(e) => log_panic!(
                                                    "Failed to get entry from {:?}: {}",
                                                    anim_path, e
                                                ),
                                            }
                                        });
                                    }
                                    Err(e) => {
                                        log_panic!("Failed to read directory {:?}: {}", anim_path, e)
                                    }
                                }

                                let anim_file = anim_path.file_name();
                                let animation_name = match anim_file.to_str() {
                                    Some(name) => name,
                                    None => log_panic!("Failed to get animation name."),
                                };

                                let mut clips = match clips.lock() {
                                    Ok(clips) => clips.clone(),
                                    Err(e) => log_panic!("Failed to lock clips: {}", e),
                                };

                                clips.par_sort();

                                app_state.loaded_animations.push(Animation {
                                    name: animation_name.to_string(),
                                    clips,
                                });
                            }
                            Err(e) => log_panic!("Failed to read SpriteInfo.json: {}", e),
                        }
                    }
                    Err(e) => log_panic!("Error while iterating path: {}", e),
                }
            }
        }
        Err(e) => log_panic!("Failed to read directory: {}", e),
    }

    app_state.loaded_collections.par_sort();
    app_state.loaded_animations.par_sort();
}

/// Packs a collection of sprites into an atlas
/// # Arguments
/// * `collection` - The collection to pack
/// * `window` - The window to send events to
/// * `sprites_path` - The path to sprite files
async fn pack_collection(
    collection: Collection,
    window: Window,
    sprites_path: String
) {
    let start = Instant::now();
    let running_task = Mutex::new(true);
    let atlas = match image::open(collection.path.clone()) {
        Ok(atlas) => atlas,
        Err(e) => log_panic!("Failed to open atlas file: {}", e),
    };
    let sprite_num = Mutex::new(0 as usize);
    let atlas_width = atlas.width();
    let atlas_height = atlas.height();
    let mut new_atlas = DynamicImage::new_rgba8(atlas_width, atlas_height);
    match new_atlas.copy_from(&atlas, 0, 0) {
        Ok(_) => info!("Successfully copied from atlas."),
        Err(e) => log_panic!("Failed to copy atlas: {}", e),
    }
    let gen_atlas = Mutex::new(new_atlas);
    collection.sprites.par_iter().try_for_each(|sprite| {
        let frame_path = match PathBuf::from_str(sprites_path.as_str()) {
            Ok(path) => path.join(sprite.path.clone()),
            Err(e) => log_panic!("Failed to create frame path from string: {}", e),
        };
        let frame_image = match image::open(frame_path.clone()) {
            Ok(image) => image,
            Err(e) => log_panic!("Failed to open frame image at {:?}: {}", frame_path.display(), e),
        };

        (0..frame_image.width() as i32).into_par_iter().try_for_each(|i| {
            (0..frame_image.height() as i32).into_par_iter().try_for_each(|j| {
                let x = if sprite.flipped {
                    sprite.x + j - sprite.yr
                } else {
                    sprite.x + i - sprite.xr
                };
                let y = if sprite.flipped {
                    atlas_width as i32 - (sprite.y + i) - 1 + sprite.xr
                } else {
                    atlas_height as i32 - (sprite.y + j) - 1 + sprite.yr
                };
                if i >= sprite.xr && i < sprite.xr + sprite.width
                    && j >= sprite.yr && j < sprite.yr + sprite.height
                    && x >= 0 && x < atlas_width as i32 && y < atlas_height as i32
                {
                    match gen_atlas.lock() {
                        Ok(mut atlas) => {
                            atlas.put_pixel(
                                x as u32,
                                cmp::max(y, 0i32) as u32,
                                frame_image.get_pixel(i as u32, frame_image.height() - j as u32 - 1),
                            );
                        }
                        Err(e) => log_panic!("Failed to lock generated atlas: {}", e),
                    }
                }

                unsafe {
                    match RX_PROGRESS.lock().expect("Failed to lock rx").as_ref() {
                        Some(rx) => {
                            match rx.try_recv() {
                                Ok(_) => {
                                    *running_task.lock().expect("Failed to lock running_task") = false;
                                    return ControlFlow::Break(());
                                },
                                Err(_) => {}
                            }
                        }
                        None => log_panic!("RX_PROGRESS is None"),
                    }
                }

                ControlFlow::Continue(())
            });

            if *running_task.lock().expect("Failed to lock running_task") == false {
                return ControlFlow::Break(());
            }

            ControlFlow::Continue(())
        });

        match sprite_num.lock() {
            Ok(mut num) => {
                *num += 1;
                window.emit("progress", ProgressPayload { progress: *num * 100 / collection.sprites.len() })
                    .expect("Failed to emit progress event");
            }
            Err(e) => log_panic!("Failed to lock sprite_num: {}", e),
        }

        if *running_task.lock().expect("Failed to lock running_task") == false {
            return ControlFlow::Break(());
        }

        ControlFlow::Continue(())
    });

    if *running_task.lock().expect("Failed to lock running_task") == false {
        return;
    }

    let stop = Instant::now();
    info!("Time to pack collection {:?}: {} ms", collection.name, stop.duration_since(start).as_millis());

    match FileDialogBuilder::new()
        .set_directory(&sprites_path)
        .set_file_name(format!("{}.png", collection.name.clone()).as_str())
        .add_filter("PNG Image", &["png"])
        .save_file() {
            Some(atlas_path) => gen_atlas.lock().expect("Failed to lock generated atlas")
                .save(atlas_path.clone()).expect("Failed to save atlas."),
            None => warn!("Generated atlas not saved.")
        }

    window.emit("enablePack", ()).expect("Failed to emit enablePack event");
}

/// Select folder containing animation files
/// # Arguments
/// * `state` - The application state
fn select_sprites_path(state: &AppState) {
    let mut app_state = state.0.lock().expect("Failed to lock app state");
    app_state.settings.sprites_path = "".to_string();
    match FileDialogBuilder::new()
        .set_directory("~")
        .pick_folder() {
            Some(folder_path) => {
                app_state.settings.sprites_path = match folder_path.into_os_string().into_string() {
                    Ok(path) => path,
                    Err(e) => log_panic!("Failed to convert path to string: {:?}", e),
                };
            }
            None => warn!("No path for sprites folder selected.")
        }

    info!("Selected sprites path as: {}", app_state.settings.sprites_path);
}

/// Begin watching sprites path for changes
/// # Arguments
/// * `state` - The application state
fn start_watcher(sprites_path: String) {
    let (tx_watcher, rx_watcher) = mpsc::channel();
    let config = notify::Config::default().with_compare_contents(true).with_poll_interval(std::time::Duration::from_secs(1));
    let mut watcher = match notify::PollWatcher::new(tx_watcher, config) {
        Ok(watcher) => watcher,
        Err(e) => log_panic!("Failed to create watcher: {}", e),
    };

    let watch_path = Path::new(&sprites_path);
    match watcher.watch(watch_path, RecursiveMode::Recursive) {
        Ok(_) => info!("Watching folder: {}", sprites_path),
        Err(e) => log_panic!("Failed to watch folder: {}", e),
    }

    loop {
        match rx_watcher.recv() {
            Ok(result) => {
                match result {
                    Ok(event) => {
                        info!("Event: {:?}", event);
                        match &event.kind {
                            EventKind::Modify(modify_kind) => {
                                match modify_kind {
                                    ModifyKind::Metadata(kind) => {
                                        for path in &event.paths {
                                            let path = path.strip_prefix(sprites_path.clone()).expect("Failed to strip prefix from path.");
                                            let path_string = match path.to_str() {
                                                Some(str) => String::from(str),
                                                None => log_panic!("Failed to convert path to string."),
                                            };
                                            let paths = path_string.split(['/', '\\']).collect::<Vec<&str>>();
                                            let sprite_info_path = PathBuf::from(sprites_path.clone()).join(paths[0]).join("0.Atlases").join("SpriteInfo.json");
                                            let collection_name = match fs::read_to_string(sprite_info_path) {
                                                Ok(text) => {
                                                    let sprite_info: SpriteInfo = match serde_json::from_str(&text) {
                                                        Ok(info) => info,
                                                        Err(e) => log_panic!("Failed to parse SpriteInfo.json: {}", e),
                                                    };
                                                    sprite_info.collection_name[0].to_string()
                                                },
                                                Err(e) => log_panic!("Failed to read SpriteInfo.json: {}", e),
                                            };
                                            if paths.len() < 3 {
                                                continue;
                                            }
                                            let sprite_name = paths[2].to_string();
                                            let sprite_data = sprite_name.split("-").collect::<Vec<&str>>();
                                            let sprite_id = sprite_data[sprite_data.len() - 1].replace(".png", "");
                                            let sprite = Sprite {
                                                id: sprite_id.to_string().parse::<u32>().expect("Failed to convert string sprite id to u32."),
                                                name: sprite_name.to_string(),
                                                collection_name: collection_name.to_string(),
                                                path: path_string,
                                                flipped: false,
                                                x: 0,
                                                y: 0,
                                                xr: 0,
                                                yr: 0,
                                                width: 0,
                                                height: 0,
                                            };

                                            unsafe {
                                                let sprites: &mut Vec<Sprite> = &mut CHANGED_SPRITES.lock().expect("Failed to lock changed sprites");
                                                if !sprites.contains(&sprite) {
                                                    sprites.push(sprite);
                                                }
                                            }
                                        }
                                    },
                                    _ => {},
                                }
                            },
                            _ => {},
                        }
                    }
                    Err(e) => warn!("Failed to receive event: {}", e.to_string()),
                }
            },
            Err(e) => warn!("Watcher error: {}", e.to_string()),
        }
    }
}

/// Cancel the currently running pack task
#[command]
fn cancel_pack() {
    unsafe {
        match &TX_PROGRESS.lock().expect("Failed to lock tx").as_ref() {
            Some(tx) => tx.send(()).expect("Failed to send cancel pack signal."),
            None => warn!("No cancel signal sent."),
        }
    }
}

/// Log a debug message
/// # Arguments
/// * `msg` - The message to log
#[command]
fn debug(msg: String) {
    info!("{}", msg);
}

/// Get an array of collections from an animation's name
/// # Arguments
/// * `animation_name` - The name of the animation
/// * `state` - The application state
/// # Returns
/// * `Vec<Collection>` - The collections used by the animation
#[command]
fn get_collections_from_animation_name(animation_name: String, state: State<AppState>) -> Vec<Collection> {
    let animation = get_animation(animation_name.clone(), state.clone());
    let mut collection_names = animation
        .clips
        .par_iter()
        .map(|clip| {
            clip.frames
                .par_iter()
                .map(|frame| frame.collection_name.clone())
        })
        .flatten() 
        .collect::<Vec<String>>();
    collection_names.par_sort();
    collection_names.dedup();
    let app_state = state.0.lock().expect("Failed to lock app state");
    let collections = collection_names
        .par_iter()
        .map(|collection_name| get_collection(collection_name.clone(), app_state.loaded_collections.clone()))
        .collect::<Vec<Collection>>();

    return collections;
}

/// Get an animation by its name
/// # Arguments
/// * `animation_name` - The name of the animation
/// * `state` - The application state
/// # Returns
/// * `Animation` - The returned animation
#[command]
fn get_animation(animation_name: String, state: State<AppState>) -> Animation {
    let app_state = state.0.lock().expect("Failed to lock app state");
    match app_state
        .loaded_animations
        .par_iter()
        .find_first(|anim| anim.name == animation_name)
    {
        Some(animation) => animation.clone(),
        None => log_panic!("Failed to find animation with name: {}", animation_name),
    }
}

/// Get an animation's name from a collection's name
/// # Arguments
/// * `collection_name` - The name of the collection
/// * `state` - The application state
/// # Returns
/// * `String` - The returned name of the animation
#[command]
fn get_animation_name_from_collection_name(collection_name: String, state: State<AppState>) -> String {
    let app_state = state.0.lock().expect("Failed to lock app state");
    let animation = match app_state.loaded_animations.par_iter().find_map_first(|anim| {
        anim.clips.par_iter().find_map_first(|clip| {
            clip.frames.par_iter().find_map_first(|frame| {
                if frame.collection_name == collection_name {
                    Some(anim)
                } else {
                    None
                }
            })
        })
    }) {
        Some(anim) => anim.clone(),
        None => log_panic!("Failed to find animation from collection name {:?}", collection_name),
    };

    animation.name
}

#[command]
fn get_collection_from_sprite_name(sprite_name: String, state: State<AppState>) -> Collection {
    let app_state = state.0.lock().expect("Failed to lock app state");
    let collection = match app_state.loaded_collections.par_iter().find_map_first(|collection| {
        collection.sprites.par_iter().find_map_first(|sprite| {
            if sprite.name == sprite_name {
                Some(collection)
            } else {
                None
            }
        })
    }) {
        Some(collection) => collection.clone(),
        None => log_panic!("Failed to find collection from sprite name {:?}", sprite_name),
    };

    collection
}

/// Get a list of animation names
/// # Arguments
/// * `state` - The application state
/// # Returns
/// * `Vec<String>` - The returned list of animation names
#[command]
fn get_animation_list(state: State<AppState>) -> Vec<String> {
    let app_state = state.0.lock().expect("Failed to lock app state");
    let animation_list = Mutex::new(Vec::new());
    let _ = &app_state.loaded_animations.par_iter().for_each(|animation| {
        match animation_list.lock() {
            Ok(mut list) => list.push(animation.name.clone()),
            Err(e) => log_panic!("Failed to lock animation list: {}", e),
        }
    });
    
    return match animation_list.lock() {
        Ok(list) => list.to_vec(),
        Err(e) => log_panic!("Failed to lock animation list: {}", e),
    };
}

/// Get the current language from settings
/// # Arguments
/// * `state` - The application state
/// # Returns
/// * `String` - The returned language
#[command]
fn get_language(state: State<AppState>) -> String {
    let app_state = state.0.lock().expect("Failed to lock app state");
    app_state.settings.language.clone()
}

/// Get the path to the sprites folder
/// # Arguments
/// * `state` - The application state
/// # Returns
/// * `String` - The returned path to the sprites folder
#[command]
fn get_sprites_path(state: State<AppState>) -> String {
    let app_state = state.0.lock().expect("Failed to lock app state");
    app_state.settings.sprites_path.clone()
}

#[command]
fn get_mode(state: State<AppState>) -> String {
    let app_state = state.0.lock().expect("Failed to lock app state");
    app_state.settings.mode.clone()
}

/// Pack a single collection
/// # Arguments
/// * `collection_name` - The name of the collection
/// * `app_handle` - The application handle
/// * `state` - The application state
#[command]
fn pack_single_collection(collection_name: String, app_handle: AppHandle, state: State<AppState>) {
    let app_state = state.0.lock().expect("Failed to lock app state");
    let collection = get_collection(collection_name.clone(), app_state.loaded_collections.clone());
    let window = match app_handle.get_window("main") {
        Some(window) => window,
        None => log_panic!("Failed to get main window"),
    };

    let sprites_path = app_state.settings.sprites_path.clone();

    unsafe {
        let (tx, rx) = mpsc::channel();
        TX_PROGRESS = Mutex::new(Some(tx));
        RX_PROGRESS = Mutex::new(Some(rx));
        async_runtime::spawn(pack_collection(collection, window, sprites_path));
    }
}

#[command]
fn set_language(language: String, menu_items: Vec<String>, app_handle: AppHandle, state: State<AppState>) {
    let mut app_state = state.0.lock().expect("Failed to lock app state");
    let menu_handle = match app_handle.get_window("main") {
        Some(window) => window.menu_handle(),
        None => log_panic!("Failed to get main window"),
    };

    let mut i = 0;

    menu_handle.get_item("quit").set_title(menu_items[i].clone()).expect("Failed to set title of Quit menu.");

    i += 1;

    menu_handle.get_item("refresh").set_title(menu_items[i].clone()).expect("Failed to set title of Refresh menu.");

    i += 1;
    
    menu_handle.get_item("set_sprites_path").set_title(menu_items[i].clone()).expect("Failed to set title of Set Sprites Path menu.");
        
    app_state.settings.language = language;
}

#[command]
fn set_mode(mode: String, state: State<AppState>) {
    let mut app_state = state.0.lock().expect("Failed to lock app state");
    app_state.settings.mode = mode.clone();
    info!("Mode set to {}", mode);
}