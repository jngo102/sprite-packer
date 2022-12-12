#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod tk2d;

use crate::app::app::App;
use crate::app::settings::Settings;
use crate::tk2d::clip::Clip;
use crate::tk2d::cln::Collection;
use crate::tk2d::info::{AnimInfo, SpriteInfo};
use crate::tk2d::lib::*;
use directories::BaseDirs;
use image::{DynamicImage, GenericImage, GenericImageView};
use log::{error, info, warn, LevelFilter};
use native_dialog::FileDialog;
use rayon::prelude::*;
use serde::Serialize;
use simple_logging;
use std::cmp;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::{async_runtime, command, Manager, State, Window};

#[derive(Clone, Serialize)]
struct ProgressPayload {
    progress: usize,
}

struct AppState(Mutex<App>);

const SETTINGS_FOLDER: &str = "SpritePacker";

/// Load the settings JSON file into the settings object, or create the file if it does not exist
/// and open the log file
/// # Arguments
/// * `state` - The state of the application
fn check_settings(state: &AppState) -> bool {
    let base_dir = BaseDirs::new().unwrap();
    let settings_dir: PathBuf = [base_dir.data_dir().to_str().unwrap(), SETTINGS_FOLDER]
        .iter()
        .collect();
    let settings_exist: bool = settings_dir.exists();
    if !settings_exist {
        match fs::create_dir(settings_dir.as_path()) {
            Ok(_) => info!("Created settings and log directory"),
            Err(e) => error!("Failed to create settings folder: {}", e),
        }
    }

    let settings_string = settings_dir.to_str().unwrap();
    let log_path = format!("{}/SpritePacker.Log", settings_string);
    match simple_logging::log_to_file(log_path.as_str(), LevelFilter::Info) {
        Ok(_) => info!("Opened logger at: {}", log_path.as_str()),
        Err(e) => {
            println!("Failed to open logger: {}", e);
            return false;
        }
    }

    let settings_path = format!("{}/Settings.json", settings_string);
    if PathBuf::from_str(settings_path.as_str()).unwrap().exists() {
        let mut app_state = state.0.lock().unwrap();
        let settings_raw_text = fs::read_to_string(settings_path).unwrap();
        app_state.settings = match serde_json::from_str(settings_raw_text.as_str()) {
            Ok(settings) => settings,
            Err(e) => {
                error!("Failed to deserialize settings: {}", e);
                Settings::default()
            }
        };
    }

    settings_exist
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
    load_collections_and_libraries(&app_state);
    let app = tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            debug,
            get_library,
            get_library_list,
            get_pack_progress,
            get_sprites_path,
            pack_library,
        ])
        .build(tauri::generate_context!())
        .expect("Failed to build tauri application.");

    app.run(move |app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();

            let app_state = app_handle.state::<AppState>();
            let state = app_state.0.lock().unwrap();
            let settings = state.settings.clone();
            let base_dir = BaseDirs::new().unwrap();
            let settings_dir: PathBuf = [base_dir.data_dir().to_str().unwrap(), SETTINGS_FOLDER]
                .iter()
                .collect();
            if !settings_dir.exists() {
                match fs::create_dir(settings_dir.as_path()) {
                    Ok(_) => info!("Succesfully created settings folder."),
                    Err(e) => error!("Failed to create settings folder: {}", e),
                }
            }
            let settings_path: PathBuf = [settings_dir.to_str().unwrap(), "Settings.json"]
                .iter()
                .collect();
            // Save or create a settings file
            if settings_path.exists() {
                let settings_file = File::options()
                    .write(true)
                    .open(settings_path.as_path())
                    .unwrap();
                match serde_json::to_writer_pretty(settings_file, &settings) {
                    Ok(_) => info!("Successfully saved settings."),
                    Err(e) => error!("Failed to save settings: {}", e),
                }
            } else {
                let mut settings_file = File::create(settings_path.as_path()).unwrap();
                let settings_string = serde_json::to_string(&state.settings).unwrap();
                match settings_file.write_all(settings_string.as_bytes()) {
                    Ok(_) => info!("Successfully created new settings file."),
                    Err(e) => error!("Failed to create new settings file: {}", e),
                }
            }

            app_handle.exit(0);
        }
        _ => {}
    });
}

fn load_collections_and_libraries(app_state: &AppState) {
    let mut state = app_state.0.lock().unwrap();
    match fs::read_dir(state.settings.sprites_path.clone()) {
        Ok(lib_paths) => {
            for lib_path in lib_paths {
                match lib_path {
                    Ok(lib_path) => {
                        let sprite_info_path =
                            lib_path.path().join("0.Atlases").join("SpriteInfo.json");
                        match fs::read_to_string(sprite_info_path) {
                            Ok(text) => {
                                let sprite_info: SpriteInfo = match serde_json::from_str(&text) {
                                    Ok(info) => info,
                                    Err(e) => panic!("Failed to parse SpriteInfo.json: {}", e),
                                };

                                for i in 0..sprite_info.id.len() {
                                    match sprite_info.at(i) {
                                        Some(sprite) => {
                                            match state.loaded_collections.par_iter().find_first(|cln| cln.name == sprite.collection_name) {
                                                Some(cln) => {
                                                    let mut cln = cln.clone();
                                                    let sprite = sprite.clone();
                                                    cln.sprites.push(sprite.clone());
                                                    (*state).loaded_collections.retain(|cln| cln.name != sprite.collection_name);
                                                    (*state).loaded_collections.push(cln);
                                                },
                                                None => {
                                                    let collection_name = sprite.clone().collection_name;
                                                    let mut cln = Collection {
                                                        name: collection_name.clone(),
                                                        path: lib_path.path().join("0.Atlases").join(format!("{}.png", collection_name)),
                                                        sprites: Vec::new(),
                                                    };
                                                    cln.sprites.push(sprite);
                                                    (*state).loaded_collections.push(cln);
                                                }
                                            }
                                        },
                                        None => panic!("Failed to get sprite at index {}.", i),
                                    }
                                }

                                let clips = Mutex::new(Vec::new());
                                match fs::read_dir(lib_path.path()) {
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
                                                        None => panic!("Failed to get file name of clip path {:?}", clip_path.path().display()),
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
                                                                                    let lib_info: AnimInfo = match serde_json::from_str(&text) {
                                                                                        Ok(lib_info) => lib_info,
                                                                                        Err(e) => panic!("Failed to parse AnimInfo.json: {}", e),
                                                                                    };
                                                                                    match fps.lock() {
                                                                                        Ok(mut fps) => *fps = lib_info.fps,
                                                                                        Err(e) => panic!("Failed to lock fps: {}", e),
                                                                                    }
                                                                                    match loop_start.lock() {
                                                                                        Ok(mut loop_start) => *loop_start = lib_info.loop_start,
                                                                                        Err(e) => panic!("Failed to lock loop_start: {}", e),
                                                                                    }
                                                                                },
                                                                                Err(e) => panic!("Failed to read AnimInfo.json: {}", e),
                                                                            }
                                                                            return;
                                                                        }
                                                                        
                                                                        let sprite = match sprite_info.path.par_iter().position_first(|path| frame_path.path().ends_with(path)) {
                                                                            Some(index) => {
                                                                                match sprite_info.at(index) {
                                                                                    Some(sprite) => sprite,
                                                                                    None => panic!("Failed to get sprite at index {}", index),
                                                                                }
                                                                            },
                                                                            None => panic!("Failed to find sprite with path {:?}", frame_path.path().display()),
                                                                        };

                                                                        match frames.lock() {
                                                                            Ok(mut frames) => frames.push(sprite),
                                                                            Err(e) => panic!("Failed to lock frames: {}", e),
                                                                        }
                                                                    },
                                                                    Err(e) => panic!("Failed to get frame path: {}", e),
                                                                }
                                                            });
                                                        }
                                                        Err(e) => panic!(
                                                            "Failed to read directory {:?}: {}",
                                                            lib_path, e
                                                        ),
                                                    }

                                                    match clip_path.file_name().to_str() {
                                                        Some(clip_name) => {
                                                            let frames = match frames.lock() {
                                                                Ok(frames) => frames.clone(),
                                                                Err(e) => panic!("Failed to lock frames: {}", e),
                                                            };
                                                            let fps = match fps.lock() {
                                                                Ok(fps) => *fps,
                                                                Err(e) => panic!("Failed to lock fps: {}", e),
                                                            };
                                                            let loop_start = match loop_start.lock() {
                                                                Ok(loop_start) => *loop_start,
                                                                Err(e) => panic!("Failed to lock loop_start: {}", e),
                                                            };
                                                            match clips.lock() {
                                                                Ok(mut clips) => clips.push(Clip::new(
                                                                    clip_name.to_string(),
                                                                    frames,
                                                                    fps,
                                                                    loop_start,
                                                                )),
                                                                Err(e) => panic!("Failed to lock clips: {}", e),
                                                            }
                                                        }
                                                        None => panic!("Failed to get clip name."),
                                                    }
                                                }
                                                Err(e) => panic!(
                                                    "Failed to get entry from {:?}: {}",
                                                    lib_path, e
                                                ),
                                            }
                                        });
                                    }
                                    Err(e) => {
                                        panic!("Failed to read directory {:?}: {}", lib_path, e)
                                    }
                                }

                                let lib_file = lib_path.file_name();
                                let library_name = match lib_file.to_str() {
                                    Some(name) => name,
                                    None => panic!("Failed to get library name."),
                                };

                                let mut clips = match clips.lock() {
                                    Ok(clips) => clips.clone(),
                                    Err(e) => panic!("Failed to lock clips: {}", e),
                                };

                                clips.par_sort();

                                (*state).loaded_libraries.push(Library {
                                    name: library_name.to_string(),
                                    clips,
                                });
                            }
                            Err(e) => panic!("Failed to read SpriteInfo.json: {}", e),
                        }
                    }
                    Err(e) => panic!("Error while iterating path: {}", e),
                }
            }
        }
        Err(e) => panic!("Failed to read directory: {}", e),
    }

    (*state).loaded_collections.par_sort();
    (*state).loaded_libraries.par_sort();
}

async fn pack_collection(
    window: Window,
    library_name: String, 
    collection_name: String, 
    sprites_path: String, 
    loaded_collections: Vec<Collection>) {
    match loaded_collections
        .par_iter()
        .find_first(|cln| cln.name == collection_name)
    {
        Some(cln) => {
            let atlas = match image::open(cln.path.clone()) {
                Ok(atlas) => atlas,
                Err(e) => panic!("Failed to open atlas file: {}", e),
            };
            let sprite_num = Mutex::new(0);
            let atlas_width = atlas.width();
            let atlas_height = atlas.height();
            let gen_atlas = Mutex::new(DynamicImage::new_rgba8(atlas_width, atlas_height));
            for sprite in cln.sprites.iter() {
                let frame_path = match PathBuf::from_str(sprites_path.as_str()) {
                    Ok(path) => path.join(sprite.path.clone()),
                    Err(e) => panic!("Failed to create frame path from string: {}", e),
                };
                let frame_image = match image::open(frame_path) {
                    Ok(image) => image,
                    Err(e) => panic!("Failed to open frame image: {}", e),
                };

                (0..frame_image.width() as i32).into_par_iter().for_each(|i| {
                    (0..frame_image.height() as i32).into_par_iter().for_each(|j| {
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
                                },
                                Err(e) => panic!("Failed to lock generated atlas: {}", e),
                            }
                        }
                    });
                });

                match sprite_num.lock() {
                    Ok(mut num) => {
                        *num += 1;
                        match window.emit("progress", ProgressPayload { progress: *num * 100 / cln.sprites.len() }) {
                            Ok(_) => info!("Emitted pack progress: {}%", *num * 100 / cln.sprites.len()),
                            Err(e) => panic!("Failed to emit progress: {}", e),
                        }
                    },
                    Err(e) => panic!("Failed to lock sprite num: {}", e),
                }
            }

            let atlas_path = match PathBuf::from_str(sprites_path.as_str()) {
                Ok(path) => path.join(library_name).join("0.Atlases").join(format!("Gen-{}.png", collection_name)),
                Err(e) => panic!("Failed to create atlas path: {}", e),
            };  

            match gen_atlas.lock() {
                Ok(atlas) => {
                    match atlas.save(atlas_path.clone()) {
                        Ok(_) => info!("Generated atlas saved to {:?}", atlas_path.display()),
                        Err(e) => panic!("Failed to save atlas: {}", e),
                    }
                },
                Err(e) => panic!("Failed to lock generated atlas: {}", e),
            };
        },
        None => panic!("Failed to find collection {}.", collection_name),
    }
}

fn select_sprites_path(app_state: &AppState) {
    let selected_path = FileDialog::new()
        .set_location("~")
        .show_open_single_dir()
        .unwrap();
    let selected_path = match selected_path {
        Some(path) => path,
        None => {
            error!("Selected path is not valid.");
            return;
        }
    };

    let mut state = app_state.0.lock().unwrap();
    state.settings.sprites_path = match selected_path.into_os_string().into_string() {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to convert path to string: {:?}", e);
            return;
        }
    };

    info!("Selected sprites path as: {}", state.settings.sprites_path);
}

#[command]
fn debug(msg: String) {
    info!("{}", msg);
}

#[command]
fn get_library(library_name: String, state: State<AppState>) -> Library {
    let app_state = state.0.lock().unwrap();
    match app_state
        .loaded_libraries
        .par_iter()
        .find_first(|lib| lib.name == library_name)
    {
        Some(library) => library.clone(),
        None => panic!("Failed to find library with name: {}", library_name),
    }
}

#[command]
fn get_library_list(state: State<AppState>) -> Vec<String> {
    let app_state = state.0.lock().unwrap();
    let library_list = Mutex::new(Vec::new());
    let _ = &app_state.loaded_libraries.par_iter().for_each(|library| {
        match library_list.lock() {
            Ok(mut list) => list.push(library.name.clone()),
            Err(e) => panic!("Failed to lock library list: {}", e),
        }
    });
    
    return match library_list.lock() {
        Ok(list) => list.to_vec(),
        Err(e) => panic!("Failed to lock library list: {}", e),
    };
}

#[command]
fn get_pack_progress(state: State<AppState>) -> usize {
    let app_state = state.0.lock().unwrap();
    info!("Current pack progress: {}", app_state.current_pack_progress);
    app_state.current_pack_progress
}

#[command]
fn get_sprites_path(state: State<AppState>) -> String {
    let app_state = state.0.lock().unwrap();
    app_state.settings.sprites_path.clone()
}

#[command]
fn pack_library(library_name: String, window: Window, state: State<AppState>) {
    let library = get_library(library_name.clone(), state.clone());
    let mut collection_names = library
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

    let mut app_state = state.0.lock().unwrap();
    app_state.current_pack_progress = 0;

    for collection_name in collection_names {
        async_runtime::spawn(
            pack_collection(
                window.clone(),
                library_name.clone(), 
                collection_name.to_string(), 
                app_state.settings.sprites_path.clone(),
                app_state.loaded_collections.clone()
            )
        );
    }
}
