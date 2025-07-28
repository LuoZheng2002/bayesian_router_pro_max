// use std::{
//     cell::RefCell,
//     sync::{Arc, Mutex},
// };

// use shared::pcb_render_model::PcbRenderModel;

// use crate::{render_context::RenderContext, render_model_to_submissions::State};

// #[derive(Default)]
// pub struct Context {
//     pub render_context: Option<RenderContext>,
//     pub state: RefCell<State>,
//     pub pcb_render_model: Arc<Mutex<Option<PcbRenderModel>>>,
//     pub working_thread: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
//     pub command_thread: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
// }
