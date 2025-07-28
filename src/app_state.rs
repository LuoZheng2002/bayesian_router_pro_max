use leptos::prelude::{ArcRwSignal, RwSignal};
use shared::pcb_render_model::PcbRenderModel;




#[derive(Clone)]
pub struct AppState {
    pub pcb_render_model: ArcRwSignal<Option<PcbRenderModel>>,
    pub render_next_frame: ArcRwSignal<bool>,
    pub step_in_disabled: ArcRwSignal<bool>,
    pub step_out_disabled: ArcRwSignal<bool>,
    pub step_over_disabled: ArcRwSignal<bool>,
    pub view_stats_disabled: ArcRwSignal<bool>,
    pub save_result_disabled: ArcRwSignal<bool>,
    pub start_pause_str: ArcRwSignal<String>,
}

impl AppState{
    pub fn new() -> Self {
        Self {
            pcb_render_model: ArcRwSignal::new(None),
            render_next_frame: ArcRwSignal::new(true),
            step_in_disabled: ArcRwSignal::new(false),
            step_out_disabled: ArcRwSignal::new(false),
            step_over_disabled: ArcRwSignal::new(false),
            view_stats_disabled: ArcRwSignal::new(false),
            save_result_disabled: ArcRwSignal::new(false),
            start_pause_str: ArcRwSignal::new("Start".into()),
        }
    }
}