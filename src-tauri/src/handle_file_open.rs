use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::{algorithm_thread, global::ALGORITHM_THREAD_HANDLE};




pub fn open_file(app_handle: AppHandle) {
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
    let file_path = app_handle
        .dialog()
        .file()
        .blocking_pick_file();
    let file_path = match file_path {
        Some(path) => path,
        None => {
            println!("No file selected");
            return;
        }
    };
    let file_path = match file_path{
        FilePath::Path(path) => path,
        FilePath::Url(_url) => {
            println!("URL files are not supported");
            return;
        }
    };
    let file_content = match std::fs::read_to_string(file_path.clone()) {
        Ok(content) => content,
        Err(err) => {
            println!("Failed to read file: {}", err);
            return;
        }
    };    
    app_handle.emit("navigate-to", "pcb").unwrap();
    println!("Emitted navigate-to event with payload: pcb");
    let mut algorithm_thread_handle = ALGORITHM_THREAD_HANDLE.lock().unwrap();
    if let Some(handle) = &mut *algorithm_thread_handle {
        {
            let mut stop_requested = handle.stop_requested.lock().unwrap();
            *stop_requested = true; // Request to stop the previous thread
        }
        handle.join_handle.take().unwrap().join().unwrap();
    }
    let stop_requested = Arc::new(Mutex::new(false));
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
}