use shared::pcb_render_model::PcbRenderModel;

use crate::global::{APP_HANDLE, CAN_SUBMIT_RENDER_MODEL, SUBMIT_RENDER_MODEL_CV};




pub fn try_submit_render_model(pcb_render_model: PcbRenderModel){
    let app_handle = {
        let app_handle = APP_HANDLE.lock().unwrap();
        app_handle.unwrap()
    };
    let can_submit_render_model = CAN_SUBMIT_RENDER_MODEL.fetch_and(false, Ordering::Relaxed);
    if can_submit_render_model{
        // submit
        app_handle.emit("submit-pcb-render-model", pcb_render_model).unwrap();
        SUBMIT_RENDER_MODEL_CV.notify_all();
    }
}
pub fn block_submit_render_model(pcb_render_model: PcbRenderModel){
    let app_handle = {
        let app_handle = APP_HANDLE.lock().unwrap();
        app_handle.unwrap()
    };
    // poll until CAN_SUBMIT_RENDER_MODEL is true
    while !CAN_SUBMIT_RENDER_MODEL.load(Ordering::Relaxed){}
    CAN_SUBMIT_RENDER_MODEL.store(false, Ordering::Relaxed);
    app_handle.emit("submit-pcb-render-model", pcb_render_model).unwrap();
    SUBMIT_RENDER_MODEL_CV.notify_all();

}