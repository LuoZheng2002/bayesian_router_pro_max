use std::sync::atomic::Ordering;

use router::command_flags::TARGET_COMMAND_LEVEL;
use shared::my_result::MyResult;
use tauri::Emitter;

use crate::global::{APP_HANDLE, COMMAND_CV};







#[tauri::command]
pub fn start_pause_click() -> MyResult<(), String> {
    // This function can be used to toggle the start/pause state
    // For now, it just returns Ok
    MyResult::Ok(())
}

#[tauri::command]
pub fn step_in()->MyResult<(), String> {
    let old_command_level = TARGET_COMMAND_LEVEL.fetch_sub(1, Ordering::SeqCst);
    if old_command_level == 0{
        TARGET_COMMAND_LEVEL.store(0, Ordering::SeqCst);
        println!("new command level is below 0, resetting to 0");
    }
    COMMAND_CV.notify_all();

    
    if old_command_level == 1 || old_command_level == 0{
        let app_handle = {
            let global_app_handle = APP_HANDLE.lock().unwrap();
            global_app_handle.clone().unwrap()
        };
        app_handle.emit("string-event", ("disable", "step-in")).unwrap();
    }    
    MyResult::Ok(())
}

#[tauri::command]
pub fn step_out()->MyResult<(), String> {
    let old_command_level = TARGET_COMMAND_LEVEL.fetch_add(1, Ordering::SeqCst);
    if old_command_level == 4{
        println!("new command level exceeds 4, resetting to 4");
        TARGET_COMMAND_LEVEL.store(4, Ordering::SeqCst);
    }
    COMMAND_CV.notify_all();
    if old_command_level == 3 || old_command_level == 4{
        let app_handle = {
            let global_app_handle = APP_HANDLE.lock().unwrap();
            global_app_handle.clone().unwrap()
        };
        app_handle.emit("string-event", ("disable", "step-out")).unwrap();
    }    
    MyResult::Ok(())
}

#[tauri::command]
pub fn step_over()->MyResult<(), String> {
    COMMAND_CV.notify_all();
    MyResult::Ok(())
}

#[tauri::command]
pub fn view_stats()->MyResult<(), String> {
    let app_handle = {
        let global_app_handle = APP_HANDLE.lock().unwrap();
        global_app_handle.clone().unwrap()
    };
    app_handle.emit("string-event", ("disable", "step-out")).unwrap();
    MyResult::Ok(())
}

#[tauri::command]
pub fn save_result()->MyResult<(), String> {
    MyResult::Ok(())
}