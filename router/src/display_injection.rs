use std::sync::{atomic::AtomicBool, Arc};

use shared::pcb_render_model::PcbRenderModel;



pub struct DisplayInjection{
    pub stop_requested: Arc<AtomicBool>,
    pub can_submit_render_model: Box<dyn FnMut() -> bool + Send>, // will set the atomic bool to false automatically
    pub submit_render_model: Box<dyn FnMut(PcbRenderModel) + Send>,
    pub block_until_signal: Box<dyn FnMut() + Send>,
}