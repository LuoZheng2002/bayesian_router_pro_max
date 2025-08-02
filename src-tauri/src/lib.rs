pub mod algorithm_thread;
pub mod handle_file_open;
pub mod global;
pub mod submit_pcb_render_model;
pub mod submission_cooldown_thread;
pub mod commands;

use std::path::Path;

use tauri::{image::Image, menu::{CheckMenuItemBuilder, IconMenuItemBuilder, Menu, MenuBuilder, SubmenuBuilder}, Emitter};
use tauri_plugin_dialog::{DialogExt, FilePath, MessageDialogButtons};

use crate::{global::{APP_HANDLE, PROGRAM_SHOULD_EXIT}};
use crate::commands::*;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

    std::thread::spawn(
        ||{
            submission_cooldown_thread::submission_cooldown_thread();
        }
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())        
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_pause,
            step_in,
            step_out,
            step_over,
            view_stats,
            save_result,
            get_settings,
            set_settings,
            get_stats,
            open_file,
        ])
        .on_window_event(|window, window_event| {
            use tauri::WindowEvent;
            if let WindowEvent::CloseRequested { api, .. } = window_event {
                PROGRAM_SHOULD_EXIT.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        })
        .setup(|app| {
            {
                let mut global_app_handle = crate::global::APP_HANDLE.lock().unwrap();
                *global_app_handle = Some(app.handle().clone());
            }

             let examples_menu = SubmenuBuilder::new(app, "Examples")
                .text("digistump_dsn", "digistump.dsn")
                .text("echo_dsn", "echo.dsn")
                .text("music_dsn", "music.dsn")
                .text("ping_dsn", "ping.dsn")
                .text("differential_dsn", "differential.dsn")
                .build()?;


            let file_menu = SubmenuBuilder::new(app, "File")
                .text("open", "Open")                
                .item(&examples_menu)
                .text("quit", "Quit")
                .build()?;

            let settings_menu = SubmenuBuilder::new(app, "Settings")
                .text("settings", "Settings")
                .build()?;

           
            
            let menu = MenuBuilder::new(app)
                .items(&[&file_menu, &settings_menu])
                .build()?;

            app.set_menu(menu)?;


            app.on_menu_event(move |app_handle: &tauri::AppHandle, event| {
                println!("menu event: {:?}", event.id());

                match event.id().0.as_str() {
                    "open" => {
                        handle_file_open::open_file(None);
                    },
                    "quit" => {
                        println!("quit event");
                        app_handle.exit(0);
                    },
                    "settings" =>{
                        println!("settings event");
                        app_handle.emit("string-event", ("navigate", "settings")).unwrap();
                    }
                    "digistump_dsn" => {
                        handle_file_open::open_file(Some("digistump.dsn".to_string()));
                    },
                    "echo_dsn" => {
                        handle_file_open::open_file(Some("echo.dsn".to_string()));
                    },
                    "music_dsn" => {
                        handle_file_open::open_file(Some("music.dsn".to_string()));
                    },
                    "ping_dsn" => {
                        handle_file_open::open_file(Some("ping.dsn".to_string()));
                    },
                    "differential_dsn" => {
                        handle_file_open::open_file(Some("differential.dsn".to_string()));
                    },
                    _ => {
                        println!("unexpected menu event");
                    }
                }
            });

            

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
