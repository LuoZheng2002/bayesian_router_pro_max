use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex};

use router::command_flags::TARGET_COMMAND_LEVEL;
use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::{algorithm_thread, global::{ALGORITHM_THREAD_HANDLE, COMMAND_CV}};




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
    app_handle.emit("string-event", ("navigate".to_string(), "pcb".to_string())).unwrap();
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
    app_handle.emit("string-event", ("start-pause".to_string(), "pause".to_string())).unwrap();
    app_handle.emit("string-event", ("disable", "step-in")).unwrap();
    app_handle.emit("string-event", ("enable", "step-over")).unwrap();
    app_handle.emit("string-event", ("enable", "step-out")).unwrap();
    app_handle.emit("string-event", ("disable", "view-stats")).unwrap();
    app_handle.emit("string-event", ("disable", "save-result")).unwrap();
}