
use std::{cell::RefCell, rc::Rc};

use leptos::{html::Canvas, prelude::*};
use leptos_router::hooks::use_navigate;
use shared::my_result::MyResult;
use tauri_sys::core::invoke;
use wasm_bindgen::{closure, prelude::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;


use crate::{app_state::{self, AppState}, render_context::RenderContext, render_model_to_submissions::render_model_to_submissions};

#[component]
pub fn PcbPage() -> impl IntoView {

    let navigate = use_navigate();
    let app_state = use_context::<AppState>().expect("AppState context not found");
    let on_settings_clicked = move |_| {
        // Handle settings button click
        // log::info!("Settings button clicked");
        navigate("/settings", Default::default());
    };

    let canvas_ref: NodeRef<Canvas> = NodeRef::<Canvas>::new();

    // We'll use this signal to track initialization status
    let (initialized, set_initialized) = signal(false);

    let on_signal_clicked = move|_|{
        spawn_local(async move {
            let result: MyResult<(), String> = invoke("signal", ()).await;
            match result{
                MyResult::Ok(_) => {
                    web_sys::console::log_1(&"Signal sent successfully".into());
                },
                MyResult::Err(e) => {
                    web_sys::console::error_1(&format!("Failed to send signal: {}", e).into());
                }
            }
        });
    };

    
    Effect::new( move |_| {
        if let Some(canvas) = canvas_ref.get() {
            if !initialized.get() {
                set_initialized.set(true);                
                // Spawn a future to handle async WGPU initialization
                let app_state = app_state.clone();
                spawn_local(async move {
                    let render_context = RenderContext::create(&canvas).await;

                    let closure_wrapper: Rc<RefCell<Option<Box<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
                    let closure_wrapper_clone = closure_wrapper.clone();
                    let render_closure: Box<dyn FnMut()> = Box::new(move ||{
                        {
                            let mut pcb_render_model = app_state.pcb_render_model.write();
                            let pcb_model = &*pcb_render_model;
                            let render_model = match pcb_model {
                            Some(model) => model,
                            None => return,
                            };
                            let render_submissions = render_model_to_submissions(render_model, &render_context);
                            render_context.render(&render_submissions).unwrap();
                            *pcb_render_model = None; // Clear the model after rendering
                        }
                        let closure_wrapper_clone = closure_wrapper_clone.clone();
                        let temp_closure = move ||{
                            let mut option_wrapper = closure_wrapper_clone.borrow_mut();
                            let wrapper = option_wrapper.as_mut().unwrap();
                            wrapper();
                        };
                        request_animation_frame(temp_closure);
                    });
                    *closure_wrapper.borrow_mut() = Some(render_closure);
                    let temp_closure = move ||{
                        let mut option_wrapper = closure_wrapper.borrow_mut();
                        let wrapper = option_wrapper.as_mut().unwrap();
                        wrapper();
                    };
                    request_animation_frame(temp_closure);
                });
            }
        }
    });



    view! {
        <div class="flex h-screen">
            // <!-- Left: Canvas -->
            <div class="flex-1 flex items-center justify-center bg-gray-100">
                <canvas id="my-canvas" width="600" height="400" class="border border-black"></canvas>
            </div>

            // <!-- Right: Column of buttons -->
            <div class="w-48 flex flex-col items-center justify-center space-y-4 bg-gray-200 p-4">
                <button on:click=on_settings_clicked class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
                    "Settings"
                </button>
                <button on:click=on_signal_clicked class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
                    "Signal"
                </button>
                <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
                    "Button 3"
                </button>
            </div>
        </div>
    }
}