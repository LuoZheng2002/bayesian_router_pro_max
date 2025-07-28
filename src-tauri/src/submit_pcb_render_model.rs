use std::sync::atomic::Ordering;

use shared::pcb_render_model::{self, PcbRenderModel};
use tauri::Emitter;

use crate::global::{APP_HANDLE, CAN_SUBMIT_RENDER_MODEL, COMMAND_CV, COMMAND_MUTEX, SUBMIT_RENDER_MODEL_CV, SUBMIT_RENDER_MODEL_MUTEX};




pub fn submit_render_model(pcb_render_model: PcbRenderModel){
    let app_handle = {
        let app_handle = APP_HANDLE.lock().unwrap();
        app_handle.clone().unwrap()
    };
    app_handle.emit("submit-pcb-render-model", pcb_render_model).unwrap();
    SUBMIT_RENDER_MODEL_CV.notify_all();
    println!("Submitted PCB render model");
}

pub fn can_submit_render_model() -> bool {
    CAN_SUBMIT_RENDER_MODEL.fetch_and(false,Ordering::Relaxed)
}

pub fn block_until_signal(){
    println!("Blocking until signal");
    let guard = COMMAND_MUTEX.lock().unwrap();
    let _unused = COMMAND_CV.wait(guard).unwrap();
    println!("Continuing");
}