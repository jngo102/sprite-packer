#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod tk2d;
mod macros;

use crate::app::app::App;
use crate::tk2d::clip::Clip;
use crate::tk2d::cln::Collection;
use crate::tk2d::info::{AnimInfo, SpriteInfo};
use crate::tk2d::anim::*;
use directories::BaseDirs;
use image::{DynamicImage, GenericImage, GenericImageView};
use log::{error, info, LevelFilter, warn};
use rayon::prelude::*;
use serde::Serialize;
use simple_logging;
use std::cmp;
use std::env;
use std::fs::{File, self};
use std::io::Write;
use std::ops::ControlFlow;
use std::path::PathBuf;
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

static mut TX: Mutex<Option<Sender<()>>> = Mutex::new(None);
static mut RX: Mutex<Option<Receiver<()>>> = Mutex::new(None);

/// The name of the folder containing the log and settings files
const SETTINGS_FOLDER: &str = "SpritePacker";

/// Load the settings JSON file into the settings object, or create the file if it does not exist
/// and open the log file
/// # Arguments
/// * `state` - The state of the application
fn check_settings(state: &AppState) -> bool {
    let base_dir = match BaseDirs::new() {
        Some(dirs) => dirs,
        None => log_panic!("Failed to get base directories"),
    };
    let data_dir = match base_dir.data_dir().to_str() {
        Some(dir) => dir,
        None => log_panic!("Failed to convert data directory to string"),
    };
    let settings_dir: PathBuf = [data_dir, SETTINGS_FOLDER]
        .iter()
        .collect();
    let settings_exist = settings_dir.exists();
    if !settings_exist {
        match fs::create_dir(settings_dir.as_path()) {
            Ok(_) => info!("Created settings and log directory"),
            Err(e) => log_panic!("Failed to create settings folder: {}", e),
        }
    }

    let log_path = settings_dir.join("SpritePacker.log");
    match simple_logging::log_to_file(log_path.clone(), LevelFilter::Info) {
        Ok(_) => info!("Opened logger at: {}", log_path.display()),
        Err(e) => {
            println!("Failed to open logger: {}", e);
            return false;
        }
    }

    let settings_path = settings_dir.join("Settings.json");
    if settings_path.exists() {
        let mut app_state = match state.0.lock() {
            Ok(state) => state,
            Err(e) => log_panic!("Failed to lock app state: {}", e),
        };
        let settings_raw_text = match fs::read_to_string(settings_path) {
            Ok(text) => text,
            Err(e) => log_panic!("Failed to read settings file: {}", e),
        };
        app_state.settings = match serde_json::from_str(settings_raw_text.as_str()) {
            Ok(settings) => settings,
            Err(e) => log_panic!("Failed to deserialize settings: {}", e),
        };
    }

    settings_exist
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
    let settings_exist = check_settings(&app_state);
    if !settings_exist {
        select_sprites_path(&app_state);
    }
    load_collections_and_animations(&app_state);

    let refresh = CustomMenuItem::new("refresh", "Refresh").accelerator("F5");
    let set_sprites_path = CustomMenuItem::new("set_sprites_path", "Set Sprites Path");
    let quit = CustomMenuItem::new("quit", "Quit").accelerator("Alt+F4");
    let submenu = Menu::new()
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
            "quit" => {
                match event.window().close() {
                    Ok(_) => info!("Successfully closed window from Options menu"),
                    Err(e) => log_panic!("Failed to close window from Options menu: {}", e),
                }
            }
            "refresh" => {
                match event.window().emit("refresh", ()) {
                    Ok(_) => info!("Successfully emitted refresh event from Options menu"),
                    Err(e) => log_panic!("Failed to emit refresh event: {}", e),
                }
            }
            "set_sprites_path" => {
                let app_handle = event.window().app_handle();
                let state = app_handle.state::<AppState>();
                select_sprites_path(&state);
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            cancel_pack,
            debug,
            get_animation,
            get_animation_name_from_clip_name,
            get_animation_name_from_collection_name,
            get_animation_list,
            get_collections_from_animation_name,
            get_clip_name_from_frame_name,
            get_language,
            get_sprites_path,
            pack_single_collection,
            set_language
        ])
        .build(tauri::generate_context!())
        .expect("Failed to build tauri application.");

    app.run(move |app_handle, event| match event {
        ExitRequested { api, .. } => {
            api.prevent_exit();

            let state = app_handle.state::<AppState>();
            let app_state = match state.0.lock() {
                Ok(state) => state,
                Err(e) => log_panic!("Failed to lock app state: {}", e),
            };
            let settings = app_state.settings.clone();
            let base_dir = BaseDirs::new().unwrap();
            let settings_dir: PathBuf = [base_dir.data_dir().to_str().unwrap(), SETTINGS_FOLDER]
                .iter()
                .collect();
            if !settings_dir.exists() {
                match fs::create_dir(settings_dir.as_path()) {
                    Ok(_) => info!("Succesfully created settings folder."),
                    Err(e) => log_panic!("Failed to create settings folder: {}", e),
                }
            }
            let settings_path = settings_dir.join("Settings.json");
            // Save or create a settings file
            if settings_path.exists() {
                let settings_file = File::options()
                    .write(true)
                    .open(settings_path.as_path())
                    .unwrap();
                match serde_json::to_writer_pretty(settings_file, &settings) {
                    Ok(_) => info!("Successfully saved settings."),
                    Err(e) => log_panic!("Failed to save settings: {}", e),
                }
            } else {
                let mut settings_file = match File::create(settings_path.as_path()) {
                    Ok(file) => file,
                    Err(e) => log_panic!("Failed to create settings file: {}", e),
                };
                let settings_string = match serde_json::to_string(&app_state.settings) {
                    Ok(settings) => settings,
                    Err(e) => log_panic!("Failed to serialize settings: {}", e),
                };
                match settings_file.write_all(settings_string.as_bytes()) {
                    Ok(_) => info!("Successfully created new settings file."),
                    Err(e) => log_panic!("Failed to create new settings file: {}", e),
                }
            }

            app_handle.exit(0);
        }
        _ => {}
    });
}

/// Load collections and animations from sprite files on disk
/// # Arguments
/// * `state` - The application state
fn load_collections_and_animations(state: &AppState) {
    let mut app_state = match state.0.lock() {
        Ok(state) => state,
        Err(e) => log_panic!("Failed to lock app state: {}", e),
    };
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
                                            match app_state.loaded_collections.par_iter().find_first(|cln| cln.name == sprite.collection_name) {
                                                Some(cln) => {
                                                    let mut cln = cln.clone();
                                                    let sprite = sprite.clone();
                                                    cln.sprites.push(sprite.clone());
                                                    (*app_state).loaded_collections.retain(|cln| cln.name != sprite.collection_name);
                                                    (*app_state).loaded_collections.push(cln);
                                                },
                                                None => {
                                                    let collection_name = sprite.clone().collection_name;
                                                    let mut cln = Collection {
                                                        name: collection_name.clone(),
                                                        path: anim_path.path().join("0.Atlases").join(format!("{}.png", collection_name)),
                                                        sprites: Vec::new(),
                                                    };
                                                    cln.sprites.push(sprite);
                                                    (*app_state).loaded_collections.push(cln);
                                                }
                                            }
                                        },
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

                                (*app_state).loaded_animations.push(Animation {
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

    (*app_state).loaded_collections.par_sort();
    (*app_state).loaded_animations.par_sort();
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
    let gen_atlas = Mutex::new(DynamicImage::new_rgba8(atlas_width, atlas_height));
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
                    match RX.lock().expect("Failed to lock rx").as_ref() {
                        Some(rx) => {
                            match rx.try_recv() {
                                Ok(_) => {
                                    *running_task.lock().expect("Failed to lock running_task") = false;
                                    return ControlFlow::Break(());
                                },
                                Err(_) => {}
                            }
                        }
                        None => log_panic!("RX is None"),
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
                match window.emit("progress", ProgressPayload { progress: *num * 100 / collection.sprites.len() }) {
                    Ok(_) => info!("Emitted progress event. Progress value: {}", *num * 100 / collection.sprites.len()),
                    Err(e) => log_panic!("Failed to emit progress event: {}", e),
                }
            }
            Err(e) => log_panic!("Failed to lock sprite_num: {}", e),
        }

        if *running_task.lock().expect("Failed to lock running_task") == false {
            return ControlFlow::Break(());
        }

        ControlFlow::Continue(())
    });

    let stop = Instant::now();
    info!("Time to pack collection {:?}: {} ms", collection.name, stop.duration_since(start).as_millis());

    match FileDialogBuilder::new()
        .set_directory(&sprites_path)
        .set_file_name(format!("{}.png", collection.name.clone()).as_str())
        .add_filter("PNG Image", &["png"])
        .save_file() {
            Some(atlas_path) => {
                match gen_atlas.lock() {
                    Ok(atlas) => {
                        match atlas.save(atlas_path.clone()) {
                            Ok(_) => info!("Generated atlas saved to {:?}", atlas_path.display()),
                            Err(e) => log_panic!("Failed to save atlas: {}", e),
                        }
                    }
                    Err(e) => log_panic!("Failed to lock generated atlas: {}", e),
                }
            }
            None => warn!("Generated atlas not saved.")
        }

    match window.emit("enablePack", ()) {
        Ok(_) => info!("Emitted enablePack event."),
        Err(e) => log_panic!("Failed to emit enablePack event: {}", e),
    }
}

/// Select folder containing animation files
/// # Arguments
/// * `state` - The application state
fn select_sprites_path(state: &AppState) {
    let mut app_state = match state.0.lock() {
        Ok(state) => state,
        Err(e) => log_panic!("Failed to lock app state: {}", e),
    };

    app_state.settings.sprites_path = "".to_string();
    while app_state.settings.sprites_path == "".to_string() {
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
        }

    info!("Selected sprites path as: {}", app_state.settings.sprites_path);
}

/// Cancel the currently running pack task
#[command]
fn cancel_pack() {
    unsafe {
        match &TX.lock() {
            Ok(tx) => match tx.as_ref() {
                Some(tx) => {
                    match tx.send(()) {
                        Ok(_) => info!("Sent cancel pack signal."),
                        Err(e) => log_panic!("Failed to send cancel pack signal: {}", e),
                    }
                }
                None => warn!("No cancel signal sent."),
            }
            Err(e) => log_panic!("Failed to lock tx: {}", e)
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
    let app_state = match state.0.lock() {
        Ok(state) => state,
        Err(e) => log_panic!("Failed to lock app state: {}", e),
    };
    let collections = collection_names
        .par_iter()
        .map(|collection_name| get_collection(collection_name.clone(), app_state.loaded_collections.clone()))
        .collect::<Vec<Collection>>();

    return collections;
}

/// Get a clip's name from a frame's name
/// # Arguments
/// * `frame_name` - The name of the frame
/// * `state` - The application state
/// # Returns
/// * `String` - The name of the clip
#[command]
fn get_clip_name_from_frame_name(frame_name: String, state: State<AppState>) -> String {
    let app_state = match state.0.lock() {
        Ok(state) => state,
        Err(e) => log_panic!("Failed to lock app state: {}", e),
    };

    let clip_index = frame_name[0..3].to_string();

    let clip = match app_state.loaded_animations.par_iter().find_map_first(|anim| {
        anim.clips.par_iter().find_first(|clip| clip.name.starts_with(&clip_index))
    }) {
        Some(clip) => clip.clone(),
        None => log_panic!("Failed to find clip with index: {}", clip_index),
    };
    
    clip.name
}

/// Get an animation by its name
/// # Arguments
/// * `animation_name` - The name of the animation
/// * `state` - The application state
/// # Returns
/// * `Animation` - The returned animation
#[command]
fn get_animation(animation_name: String, state: State<AppState>) -> Animation {
    let app_state = match state.0.lock() {
        Ok(state) => state,
        Err(e) => log_panic!("Failed to lock app state: {}", e),
    };
    match app_state
        .loaded_animations
        .par_iter()
        .find_first(|anim| anim.name == animation_name)
    {
        Some(animation) => animation.clone(),
        None => log_panic!("Failed to find animation with name: {}", animation_name),
    }
}

/// Get an animation's name from a clip's name
/// # Arguments
/// * `clip_name` - The name of the clip
/// * `state` - The application state
/// # Returns
/// * `String` - The name of the animation
#[command]
fn get_animation_name_from_clip_name(clip_name: String, state: State<AppState>) -> String {
    let app_state = match state.0.lock() {
        Ok(state) => state,
        Err(e) => log_panic!("Failed to lock app state: {}", e),
    };
    let animation = match app_state
        .loaded_animations
        .par_iter()
        .find_first(|anim| match anim.clips.par_iter().find_first(|clip| clip.name == clip_name) {
            Some(_) => true,
            None => false,
        })
    {
        Some(anim) => anim,
        None => log_panic!("Failed to find animation from clip name {:?}", clip_name),
    };
    
    return animation.name.clone();
}

/// Get an animation's name from a collection's name
/// # Arguments
/// * `collection_name` - The name of the collection
/// * `state` - The application state
/// # Returns
/// * `String` - The returned name of the animation
#[command]
fn get_animation_name_from_collection_name(collection_name: String, state: State<AppState>) -> String {
    let app_state = match state.0.lock() {
        Ok(state) => state,
        Err(e) => log_panic!("Failed to lock app state: {}", e),
    };
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

/// Get a list of animation names
/// # Arguments
/// * `state` - The application state
/// # Returns
/// * `Vec<String>` - The returned list of animation names
#[command]
fn get_animation_list(state: State<AppState>) -> Vec<String> {
    let app_state = match state.0.lock() {
        Ok(state) => state,
        Err(e) => log_panic!("Failed to lock app state: {}", e),
    };
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
    let app_state = match state.0.lock() {
        Ok(state) => state,
        Err(e) => log_panic!("Failed to lock app state: {}", e),
    };
    app_state.settings.language.clone()
}

/// Get the path to the sprites folder
/// # Arguments
/// * `state` - The application state
/// # Returns
/// * `String` - The returned path to the sprites folder
#[command]
fn get_sprites_path(state: State<AppState>) -> String {
    let app_state = match state.0.lock() {
        Ok(state) => state,
        Err(e) => log_panic!("Failed to lock app state: {}", e),
    };
    app_state.settings.sprites_path.clone()
}

/// Pack a single collection
/// # Arguments
/// * `collection_name` - The name of the collection
/// * `app_handle` - The application handle
/// * `state` - The application state
#[command]
fn pack_single_collection(collection_name: String, app_handle: AppHandle, state: State<AppState>) {
    let app_state = match state.0.lock() {
        Ok(state) => state,
        Err(e) => log_panic!("Failed to lock app state: {}", e),
    };
    let collection = get_collection(collection_name.clone(), app_state.loaded_collections.clone());
    let window = match app_handle.get_window("main") {
        Some(window) => window,
        None => log_panic!("Failed to get main window"),
    };

    let sprites_path = app_state.settings.sprites_path.clone();

    unsafe {
        let (tx, rx) = mpsc::channel();
        TX = Mutex::new(Some(tx));
        RX = Mutex::new(Some(rx));
        async_runtime::spawn(pack_collection(collection, window, sprites_path));
    }
}

#[command]
fn set_language(language: String, menu_items: Vec<String>, app_handle: AppHandle, state: State<AppState>) {
    let mut app_state = match state.0.lock() {
        Ok(state) => state,
        Err(e) => log_panic!("Failed to lock app state: {}", e),
    };

    let menu_handle = match app_handle.get_window("main") {
        Some(window) => window.menu_handle(),
        None => log_panic!("Failed to get main window"),
    };

    let mut i = 0;

    match menu_handle.get_item("quit").set_title(menu_items[i].clone()) {
        Ok(_) => info!("Quit label is now {}", menu_items[i].clone()),
        Err(e) => log_panic!("Failed to set title of Quit menu to {}: {}", menu_items[i].clone(), e),
    }

    i += 1;

    match menu_handle.get_item("refresh").set_title(menu_items[i].clone()) {
        Ok(_) => info!("Refresh label is now {}", menu_items[i].clone()),
        Err(e) => log_panic!("Failed to set title of Refresh menu to {}: {}", menu_items[i].clone(), e),
    }

    i += 1;
    
    match menu_handle.get_item("set_sprites_path").set_title(menu_items[i].clone()) {
        Ok(_) => info!("Set Sprites Path label is now {}", menu_items[i].clone()),
        Err(e) => log_panic!("Failed to set title of Set Sprites Path menu to {}: {}", menu_items[i].clone(), e),
    }
        
    app_state.settings.language = language;
}