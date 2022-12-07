#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod anim;
mod app;

use crate::anim::anim::{AnimInfo, Animation};
use crate::anim::cln::Collection;
use crate::anim::sprite::Sprite;
use crate::app::app::App;
use crate::app::settings::Settings;
use directories::BaseDirs;
use eventual::Timer;
use log::{error, info, warn, LevelFilter};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use simple_logging;
use std::env;
use std::fs;
use std::fs::{File, ReadDir};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{mpsc, Mutex, MutexGuard};
use std::thread;
use tauri::{async_runtime, command, Manager, State};

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
        let state = app_state.0.lock().unwrap();
        select_sprites_path(state);
    }
    let app = tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            debug,
            get_collection,
            get_collection_list,
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

fn select_sprites_path(mut app: MutexGuard<App>) {
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

    (*app).settings.sprites_path = match selected_path.into_os_string().into_string() {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to convert path to string: {:?}", e);
            return;
        }
    };

    info!("Selected sprites path as: {}", app.settings.sprites_path);
}

#[command]
fn debug(msg: String) {
    info!("{}", msg);
}

#[command]
fn get_collection(collection_name: String, state: State<AppState>) -> Collection {
    let app_state = state.0.lock().unwrap();
    match fs::read_dir(app_state.settings.sprites_path.clone()) {
        Ok(collection_paths) => {
            for collection_path in collection_paths {
                match collection_path {
                    Ok(collection_path) => {
                        let folder_name = match collection_path.file_name().into_string() {
                            Ok(folder_name) => folder_name,
                            Err(e) => panic!("Failed to convert path to string: {:?}", e),
                        };
                        if folder_name == collection_name {
                            let mut animations = Vec::new();
                            match fs::read_dir(collection_path.path()) {
                                Ok(anim_paths) => {
                                    for anim_path in anim_paths {
                                        match anim_path {
                                            Ok(anim_path) => {
                                                if anim_path.file_name() == "0.Atlases" {
                                                    continue;
                                                }
                                                let mut frames = Vec::new();
                                                let mut fps = 0.0;
                                                let mut loop_start = 0;
                                                match fs::read_dir(anim_path.path()) {
                                                    Ok(frame_paths) => {
                                                        for frame_path in frame_paths {
                                                            match frame_path {
                                                        Ok(frame_path) => {
                                                            if frame_path.file_name() == "AnimInfo.json" {
                                                                match fs::read_to_string(frame_path.path()) {
                                                                    Ok(text) => {
                                                                        let anim_info: AnimInfo = match serde_json::from_str(&text) {
                                                                            Ok(anim_info) => anim_info,
                                                                            Err(e) => panic!("Failed to parse AnimInfo.json: {}", e),
                                                                        };
                                                                        fps = anim_info.fps;
                                                                        loop_start = anim_info.loop_start;
                                                                    },
                                                                    Err(e) => panic!("Failed to read AnimInfo.json: {}", e),
                                                                }
                                                                continue;
                                                            }
                                                            match frame_path.file_name().into_string() {
                                                                Ok(frame_name) => frames.push(frame_name),
                                                                Err(e) => panic!("Failed to get frame name from file name: {:?}", e),
                                                            }
                                                        },
                                                        Err(e) => panic!("Failed to read get frame path: {}", e),
                                                    }
                                                        }
                                                    }
                                                    Err(e) => panic!(
                                                        "Failed to read directory {:?}: {}",
                                                        anim_path, e
                                                    ),
                                                }
                                                let anim_name = match anim_path.file_name().into_string() {
                                                    Ok(anim_name) => anim_name,
                                                    Err(e) => panic!("Failed to get animation name from file name: {:?}", e),
                                                };
                                                animations.push(Animation::new(
                                                    anim_name, frames, fps, loop_start,
                                                ));
                                            }
                                            Err(e) => panic!(
                                                "Failed to read directory {:?}: {}",
                                                folder_name, e
                                            ),
                                        }
                                    }
                                }
                                Err(e) => {
                                    panic!("Failed to read directory {:?}: {}", folder_name, e)
                                }
                            };
                            info!("Returning collection: {}", collection_name);
                            return Collection {
                                name: collection_name,
                                animations,
                            };
                        }
                    }
                    Err(e) => panic!("Error while iterating path: {}", e),
                }
            }
        }
        Err(e) => panic!("Failed to read directory: {}", e),
    }

    panic!("Failed to find collection {:?}", collection_name);
}

#[command]
fn get_collection_list(state: State<AppState>) -> Vec<String> {
    info!("Called twice??");
    let app_state = state.0.lock().unwrap();
    let mut collections = Vec::new();
    match fs::read_dir(app_state.settings.sprites_path.clone()) {
        Ok(collection_paths) => {
            for collection_path in collection_paths {
                match collection_path {
                    Ok(collection_path) => {
                        match collection_path.file_name().into_string() {
                            Ok(collection_name) => {
                                collections.push(collection_name);
                            },
                            Err(e) => panic!("Failed to convert path to string: {:?}", e),
                        }
                    },
                    Err(e) => panic!("Failed to get collection path: {}", e),
                }
            }
        },
        Err(e) => panic!("Failed to read directory: {}", e),
    }

    let mut i = 0;
    for collection in collections.clone() {
        info!("Collection name {}: {:?}", i, collection);
        i += 1;
    }
    collections
}