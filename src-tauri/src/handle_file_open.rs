use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex};

use router::command_flags::TARGET_COMMAND_LEVEL;
use shared::my_result::MyResult;
use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::{algorithm_thread, global::{ALGORITHM_THREAD_HANDLE, APP_HANDLE, COMMAND_CV}};




pub fn open_file()->MyResult<(), String> {
    // Handle the file open event
    println!("open event");
    // let randomize_algorithm = app_handle
    //     .dialog()
    //     .message("是否随机化未定义的算法？（用于调试）")
    //     .title("加载模型")
    //     .buttons(MessageDialogButtons::OkCancelCustom(
    //         "随机化".to_string(),
    //         "不随机化".to_string(),
    //     ))
    //     .blocking_show();        
    let (file_path, file_content) = {
        let app_handle = crate::global::APP_HANDLE.lock().unwrap();
        let app_handle = app_handle.clone().unwrap();
        let file_path = app_handle
        .dialog()
        .file()
        .blocking_pick_file();
        let file_path = match file_path {
            Some(path) => path,
            None => {
                println!("No file selected");
                return MyResult::Err("No file selected".to_string());
            }
        };
        let file_path = match file_path{
            FilePath::Path(path) => path,
            FilePath::Url(_url) => {
                println!("URL files are not supported");
                return MyResult::Err("URL files are not supported".to_string());
            }
        };
        let file_content = match std::fs::read_to_string(file_path.clone()) {
            Ok(content) => content,
            Err(err) => {
                println!("Failed to read file: {}", err);
                return MyResult::Err(format!("Failed to read file: {}", err));
            }
        };   
        app_handle.emit("string-event", ("navigate".to_string(), "pcb".to_string())).unwrap();
        (file_path, file_content)
    };    
    println!("Emitted string-event with payload: navigate pcb");
    let mut algorithm_thread_handle = ALGORITHM_THREAD_HANDLE.lock().unwrap();
    if let Some(handle) = &mut *algorithm_thread_handle {
        handle.stop_requested.store(true, Ordering::Relaxed);
        COMMAND_CV.notify_all();
        handle.join_handle.take().unwrap().join().unwrap();
    }
    let stop_requested = Arc::new(AtomicBool::new(false));
    let stop_requested_clone = stop_requested.clone();
    let file_path_clone = file_path.clone();
    println!("Starting an algorithm thread");
    let new_join_handle = std::thread::spawn(move || {
        algorithm_thread::algorithm_thread(file_path_clone, file_content, stop_requested_clone);
    });
    *algorithm_thread_handle = Some(algorithm_thread::AlgorithmThreadHandle {
        stop_requested,
        join_handle: Some(new_join_handle),
        file_path,
    });
    // disable step in, step out, step over buttons
    TARGET_COMMAND_LEVEL.store(0, Ordering::Relaxed);
    println!("Initializing buttons");
    // std::thread::sleep(std::time::Duration::from_millis(2000));
    let app_handle = {
        let app_handle = APP_HANDLE.lock().unwrap();
        app_handle.clone().unwrap()
    };
    app_handle.emit("string-event", ("start-pause".to_string(), "pause".to_string())).unwrap();
    app_handle.emit("string-event", ("disable".to_string(), "step-in".to_string())).unwrap();
    app_handle.emit("string-event", ("enable".to_string(), "step-over".to_string())).unwrap();
    println!("At the middle of initializing buttons");
    app_handle.emit("string-event", ("enable".to_string(), "step-out".to_string())).unwrap();
    // app_handle.emit("string-event", ("disable".to_string(), "view-stats".to_string())).unwrap();
    // app_handle.emit("string-event", ("disable".to_string(), "save-result".to_string())).unwrap();
    println!("Finished initializing buttons");
    MyResult::Ok(())
}