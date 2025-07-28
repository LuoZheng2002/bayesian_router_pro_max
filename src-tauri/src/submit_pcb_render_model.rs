use std::sync::atomic::Ordering;

use shared::pcb_render_model::{self, PcbRenderModel};
use tauri::Emitter;

use crate::global::{APP_HANDLE, CAN_SUBMIT_RENDER_MODEL, SUBMIT_RENDER_MODEL_CV, SUBMIT_RENDER_MODEL_MUTEX};




// pub fn try_submit_render_model(render_model_generator: Box<dyn FnOnce() -> PcbRenderModel + Send>){
//     let app_handle = {
//         let app_handle = APP_HANDLE.lock().unwrap();
//         app_handle.clone().unwrap()
//     };
//     let can_submit_render_model = CAN_SUBMIT_RENDER_MODEL.fetch_and(false, Ordering::Relaxed);
//     if can_submit_render_model{
//         let pcb_render_model = render_model_generator();
//         app_handle.emit("submit-pcb-render-model", pcb_render_model).unwrap();
//         SUBMIT_RENDER_MODEL_CV.notify_all();
//     }
// }
// pub fn block_submit_render_model(pcb_render_model: PcbRenderModel){
//     let app_handle = {
//         let app_handle = APP_HANDLE.lock().unwrap();
//         app_handle.clone().unwrap()
//     };
//     // poll until CAN_SUBMIT_RENDER_MODEL is true
//     while !CAN_SUBMIT_RENDER_MODEL.load(Ordering::Relaxed){}
//     CAN_SUBMIT_RENDER_MODEL.store(false, Ordering::Relaxed);
//     app_handle.emit("submit-pcb-render-model", pcb_render_model).unwrap();
//     SUBMIT_RENDER_MODEL_CV.notify_all();
// }

pub fn submit_render_model(pcb_render_model: PcbRenderModel){
    let app_handle = {
        let app_handle = APP_HANDLE.lock().unwrap();
        app_handle.clone().unwrap()
    };
    app_handle.emit("submit-pcb-render-model", pcb_render_model).unwrap();
}

pub fn can_submit_render_model() -> bool {
    CAN_SUBMIT_RENDER_MODEL.fetch_and(false,Ordering::Relaxed)
}

pub fn block_until_signal(){
    let guard = SUBMIT_RENDER_MODEL_MUTEX.lock().unwrap();
    let _unused = SUBMIT_RENDER_MODEL_CV.wait(guard).unwrap();
}