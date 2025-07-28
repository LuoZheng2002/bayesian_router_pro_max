pub mod algorithm_thread;
pub mod handle_file_open;
pub mod global;
pub mod submit_pcb_render_model;
pub mod submission_cooldown_thread;
pub mod commands;

use std::path::Path;

use tauri::{image::Image, menu::{CheckMenuItemBuilder, IconMenuItemBuilder, Menu, MenuBuilder, SubmenuBuilder}, Emitter};
use tauri_plugin_dialog::{DialogExt, FilePath, MessageDialogButtons};

use crate::{global::{APP_HANDLE, PROGRAM_SHOULD_EXIT}, handle_file_open::open_file};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())        
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet])
        .on_window_event(|window, window_event| {
            use tauri::WindowEvent;
            if let WindowEvent::CloseRequested { api, .. } = window_event {
                PROGRAM_SHOULD_EXIT.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        })
        .setup(|app| {
            let file_menu = SubmenuBuilder::new(app, "File")
                .text("open", "Open")
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
                {
                    let mut global_app_handle = APP_HANDLE.lock().unwrap();
                    *global_app_handle = Some(app_handle.clone());
                }
                println!("menu event: {:?}", event.id());

                match event.id().0.as_str() {
                    "open" => {
                        open_file(app_handle.clone());
                    }
                    "quit" => {
                        println!("quit event");
                        app_handle.exit(0);
                    }
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
