
use std::{cell::RefCell, rc::Rc};

use leptos::{html::Canvas, prelude::*};
use leptos_router::hooks::use_navigate;
use leptos_use::{use_resize_observer, UseResizeObserverReturn};
use shared::my_result::MyResult;
use tauri_sys::core::invoke;
use wasm_bindgen::{closure, prelude::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, ResizeObserver, ResizeObserverEntry};


use crate::{app_state::{self, AppState}, render_context::{self, RenderContext}, render_model_to_submissions::render_model_to_submissions};

#[component]
pub fn PcbPage() -> impl IntoView {

    let navigate = use_navigate();
    let app_state = use_context::<AppState>().expect("AppState context not found");
    let app_state2 = app_state.clone();
    let app_state3 = app_state.clone();
    let app_state4 = app_state.clone();
    let app_state5 = app_state.clone();

    // let (hint_msg, set_hint_msg) = signal("This is a hint message".to_string());


    // let on_settings_clicked = move |_| {
    //     // Handle settings button click
    //     // log::info!("Settings button clicked");
    //     navigate("/settings", Default::default());
    // };

    // let (start_pause_str, set_start_pause_str) = signal("Start");

    let canvas_ref: NodeRef<leptos::html::Canvas> = NodeRef::new();

    // We'll use this signal to track initialization status
    let (initialized, set_initialized) = signal(false);
    let (render_context, set_render_context) = signal_local::<Rc<RefCell<Option<RenderContext>>>>(Rc::new(RefCell::new(None)));

    // let on_signal_clicked = move|_|{
    //     spawn_local(async move {
    //         let result: MyResult<(), String> = invoke("signal", ()).await;
    //         match result{
    //             MyResult::Ok(_) => {
    //                 web_sys::console::log_1(&"Signal sent successfully".into());
    //             },
    //             MyResult::Err(e) => {
    //                 web_sys::console::error_1(&format!("Failed to send signal: {}", e).into());
    //             }
    //         }
    //     });
    // };
    use_resize_observer(canvas_ref.clone(), 
        move |entries: Vec<ResizeObserverEntry>, _observer: ResizeObserver| {
        for entry in entries {
            let content_rect = entry.content_rect();
            web_sys::console::log_1(&format!(
                "Resized: width = {}, height = {}",
                content_rect.width(),
                content_rect.height()
            ).into());
            let render_context = render_context.get();
            let mut render_context = render_context.borrow_mut();
            if let Some(render_context) = render_context.as_mut() {
                render_context.resize((content_rect.width() as u32, content_rect.height() as u32));
                let mut render_next_frame = app_state2.render_next_frame.write();
                *render_next_frame = true; // Trigger a render on resize
            }
        }
    });

    
    Effect::new( move |_| {
        if let Some(canvas) = canvas_ref.get() {
            if !initialized.get() {
                set_initialized.set(true);                
                // Spawn a future to handle async WGPU initialization
                let app_state = app_state.clone();
                spawn_local(async move {
                    // let render_context = RenderContext::create(&canvas).await;
                    let temp_render_context = RenderContext::create(&canvas).await;

                    temp_render_context.resize((800, 600));
                    set_render_context.set(Rc::new(RefCell::new(Some(temp_render_context))));
                    let closure_wrapper: Rc<RefCell<Option<Box<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
                    let closure_wrapper_clone = closure_wrapper.clone();
                    let render_closure: Box<dyn FnMut()> = Box::new(move ||{
                        let render_context = render_context.get_untracked();
                        let render_context = render_context.borrow_mut();
                        let render_context = render_context.as_ref().unwrap();
                        let try_render_pcb = ||{
                            let mut render_next_frame = app_state.render_next_frame.write();
                            let pcb_render_model = app_state.pcb_render_model.write();
                            let pcb_model = &*pcb_render_model;
                            if *render_next_frame{
                                *render_next_frame = false; // Reset the flag
                                let render_model = match pcb_model {
                                    Some(model) => model,
                                    None => return,
                                };
                                web_sys::console::log_1(&format!("render model is not none, trying to render a frame").into());
                                let render_submissions = render_model_to_submissions(render_model, &render_context);
                                render_context.render(&render_submissions).unwrap();
                            }                            
                            // *pcb_render_model = None; // Clear the model after rendering
                        };
                        try_render_pcb();
                        let closure_wrapper_clone = closure_wrapper_clone.clone();
                        let temp_closure = move ||{
                            let mut option_wrapper = closure_wrapper_clone.borrow_mut();
                            let wrapper = option_wrapper.as_mut().unwrap();
                            wrapper();
                        };
                        // web_sys::console::log_1(&"Requesting next animation frame".into());
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

    let on_start_pause = move |_| {
        // app_state.increase_command_level();// to do
        spawn_local(async move {
            let result: MyResult<(), String> = invoke("start_pause", ()).await;
            match result {
                MyResult::Ok(_) => {
                    web_sys::console::log_1(&"Start/Pause command executed successfully".into());
                },
                MyResult::Err(e) => {
                    web_sys::console::error_1(&format!("Failed to execute Start/Pause command: {}", e).into());
                }
            }
        });
    };


    let on_step_in = move |_| {
        // app_state.increase_command_level();// to do
        spawn_local(async move {
            let result: MyResult<(), String> = invoke("step_in", ()).await;
            match result {
                MyResult::Ok(_) => {
                    web_sys::console::log_1(&"Step In command executed successfully".into());
                },
                MyResult::Err(e) => {
                    web_sys::console::error_1(&format!("Failed to execute Step In command: {}", e).into());
                }
            }
        });
    };
    let on_step_over = move |_| {
        spawn_local(async move {
            let result: MyResult<(), String> = invoke("step_over", ()).await;
            match result {
                MyResult::Ok(_) => {
                    web_sys::console::log_1(&"Step Over command executed successfully".into());
                },
                MyResult::Err(e) => {
                    web_sys::console::error_1(&format!("Failed to execute Step Over command: {}", e).into());
                }
            }
        });
    };
    let on_step_out = move |_| {
        spawn_local(async move {
            let result: MyResult<(), String> = invoke("step_out", ()).await;
            match result {
                MyResult::Ok(_) => {
                    web_sys::console::log_1(&"Step Out command executed successfully".into());
                },
                MyResult::Err(e) => {
                    web_sys::console::error_1(&format!("Failed to execute Step Out command: {}", e).into());
                }
            }
        });
    };

    let on_view_stats = move |_| {
        navigate("/stats", Default::default());
        // spawn_local(async move {
        //     let result: MyResult<(), String> = invoke("view_stats", ()).await;
        //     match result {
        //         MyResult::Ok(_) => {
        //             web_sys::console::log_1(&"View Stats command executed successfully".into());
        //         },
        //         MyResult::Err(e) => {
        //             web_sys::console::error_1(&format!("Failed to execute View Stats command: {}", e).into());
        //         }
        //     }
        // });
    };

    let on_save_result = move |_| {
        let app_state4 = app_state4.clone();
        spawn_local(async move {
            let result: MyResult<(), String> = invoke("save_result", ()).await;
            match result {
                MyResult::Ok(_) => {
                    app_state4.hint_message.set("Saved Successfully".to_string());
                    web_sys::console::log_1(&"Save Result command executed successfully".into());
                },
                MyResult::Err(e) => {
                    app_state4.hint_message.set(format!("Failed to save result: {}", e));
                    web_sys::console::error_1(&format!("Failed to execute Save Result command: {}", e).into());
                }
            }
        });
    };


    view! {
        <div class="flex h-screen">
            // <!-- Left: Canvas -->
            <div class="flex-1 flex items-center justify-center bg-gray-100">
                <canvas
                    id="my-canvas"
                    node_ref=canvas_ref
                    class="border border-black w-[80vw] h-[80vh]"
                ></canvas>
            </div>

            // <!-- Right: Column of buttons -->
            <div class="w-48 flex flex-col items-center justify-center space-y-4 bg-gray-200 p-4">
                <button
                    on:click=on_start_pause
                    class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                >
                    {app_state3.start_pause_str}
                </button>
                <button
                    on:click=on_step_in
                    disabled=app_state3.step_in_disabled
                    class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                >
                    "Step In"
                </button>
                <button
                    on:click=on_step_over
                    disabled=app_state3.step_over_disabled
                    class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                >
                    "Step Over"
                </button>
                <button
                    on:click=on_step_out
                    disabled=app_state3.step_out_disabled
                    class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                >
                    "Step Out"
                </button>
                <button
                    on:click=on_view_stats
                    disabled=app_state3.view_stats_disabled
                    class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                >
                    "View Stats"
                </button>
                <button
                    on:click=on_save_result
                    disabled=app_state3.save_result_disabled
                    class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                >
                    "Save Result"
                </button>
            </div>
            <div
                class="fixed bottom-1 left-1/2 transform -translate-x-1/2 z-50 flex items-start gap-3 p-4 bg-red-100 border border-red-400 text-red-700 rounded-lg shadow-md animate-fade-in"
                role="alert"
            >
                <svg
                    class="w-6 h-6 text-red-500 mt-1"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M12 8v4m0 4h.01M21 12A9 9 0 113 12a9 9 0 0118 0z"
                    />
                </svg>
                <span class="text-sm font-medium">{app_state5.hint_message}</span>
            </div>
        </div>
    }
}