use leptos::prelude::ArcRwSignal;
use shared::pcb_render_model::PcbRenderModel;




#[derive(Clone)]
pub struct AppState {
    pub pcb_render_model: ArcRwSignal<Option<PcbRenderModel>>,
}

impl AppState{
    pub fn new() -> Self {
        Self {
            pcb_render_model: ArcRwSignal::new(None),
        }
    }
}