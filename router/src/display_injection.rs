use shared::pcb_render_model::PcbRenderModel;



pub struct DisplayInjection{
    pub can_submit_render_model: Box<dyn FnMut() -> bool + Send>, // will set the atomic bool to false automatically
    pub submit_render_model: Box<dyn FnMut(PcbRenderModel) + Send>,
    pub block_until_signal: Box<dyn FnMut() + Send>,
}