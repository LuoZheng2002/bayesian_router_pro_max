use leptos::{prelude::*, reactive::spawn_local};
use leptos_router::hooks::use_navigate;
use serde_wasm_bindgen::to_value;
use shared::{my_result::MyResult, settings_enum::{GetSettingsArg, SetSettingsArg, SettingsEnum}};
use tauri_sys::core::invoke;

// use bayesian inference    bool
// astar max expansions     usize >=1 recommended      
// astar stride f32 >= 0.01, recommended 1.27
// trace score causing probability halved   f64 >=0.1  recommended 10
// opportunity cost causing probability halved  f64 >= 0.1 recommended 0.5
// max trace generation attempts usize >= 1, recommended 4
// first iteration prior probability f64 > 0.0, recommended 0.5
// second iteration prior probability f64 > 0.0, recommended 0.4
// second iteration num traces usize >= 1, recommended 3
// via cost (mm) f64 >= 0.0, recommended 5.0

// bayesian inference related:
// num top ranked to try     usize >= 1, recommended 3
// sample iterations (1 or 2) usize 1 or 2 recommended 2   
// update probability skip stride usize >= 1, recommended 2

#[component]
pub fn SettingsPage() -> impl IntoView {
    let navigate = use_navigate();

    
    let (initialized, set_initialized) = signal(false);

    let (use_bayesian_inference, set_use_bayesian_inference) = signal(false);
    let (astar_max_expansions, set_astar_max_expansions) = signal::<usize>(0);
    let (astar_stride, set_astar_stride) = signal::<f64>(0.0);
    let (trace_score_halved, set_trace_score_halved) = signal::<f64>(0.0);
    let (opportunity_cost_halved, set_opportunity_cost_halved) = signal::<f64>(0.0);
    let (max_trace_generation_attempts, set_max_trace_generation_attempts) = signal::<usize>(0);
    let (first_iteration_prior_probability, set_first_iteration_prior_probability) = signal::<f64>(0.0);
    let (second_iteration_prior_probability, set_second_iteration_prior_probability) = signal::<f64>(0.0);
    let (second_iteration_num_traces, set_second_iteration_num_traces) = signal::<usize>(0);
    let (via_cost, set_via_cost) = signal::<f64>(0.0);
    let (num_top_ranked_to_try, set_num_top_ranked_to_try) = signal::<usize>(0);
    let (sample_iterations, set_sample_iterations) = signal::<usize>(0);
    let (update_probability_skip_stride, set_update_probability_skip_stride) = signal::<usize>(0);

    let on_back_clicked = move |_|{
        let navigate = navigate.clone();
        spawn_local(async move{
            let result = invoke::<MyResult<(), String>>("set_settings", SetSettingsArg::new("use_bayesian_inference".into(), SettingsEnum::Bool(use_bayesian_inference.get_untracked()))).await;
            web_sys::console::log_1(&format!("result: {:?}", result).into());
            let result = invoke::<MyResult<(), String>>("set_settings", SetSettingsArg::new("astar_max_expansions".into(), SettingsEnum::Usize(astar_max_expansions.get_untracked()))).await;
            web_sys::console::log_1(&format!("result: {:?}", result).into());
            let result = invoke::<MyResult<(), String>>("set_settings", SetSettingsArg::new("astar_stride".into(), SettingsEnum::Float(astar_stride.get_untracked()))).await;
            web_sys::console::log_1(&format!("result: {:?}", result).into());
            let result = invoke::<MyResult<(), String>>("set_settings", SetSettingsArg::new("trace_score_halved".into(), SettingsEnum::Float(trace_score_halved.get_untracked()))).await;
            web_sys::console::log_1(&format!("result: {:?}", result).into());
            let result = invoke::<MyResult<(), String>>("set_settings", SetSettingsArg::new("opportunity_cost_halved".into(), SettingsEnum::Float(opportunity_cost_halved.get_untracked()))).await;
            web_sys::console::log_1(&format!("result: {:?}", result).into());
            let result = invoke::<MyResult<(), String>>("set_settings", SetSettingsArg::new("max_trace_generation_attempts".into(), SettingsEnum::Usize(max_trace_generation_attempts.get_untracked()))).await;
            web_sys::console::log_1(&format!("result: {:?}", result).into());
            let result = invoke::<MyResult<(), String>>("set_settings", SetSettingsArg::new("first_iteration_prior_probability".into(), SettingsEnum::Float(first_iteration_prior_probability.get_untracked()))).await;
            web_sys::console::log_1(&format!("result: {:?}", result).into());
            let result = invoke::<MyResult<(), String>>("set_settings", SetSettingsArg::new("second_iteration_prior_probability".into(), SettingsEnum::Float(second_iteration_prior_probability.get_untracked()))).await;
            web_sys::console::log_1(&format!("result: {:?}", result).into());
            let result = invoke::<MyResult<(), String>>("set_settings", SetSettingsArg::new("second_iteration_num_traces".into(), SettingsEnum::Usize(second_iteration_num_traces.get_untracked()))).await;
            web_sys::console::log_1(&format!("result: {:?}", result).into());
            let result = invoke::<MyResult<(), String>>("set_settings", SetSettingsArg::new("via_cost".into(), SettingsEnum::Float(via_cost.get_untracked()))).await;
            web_sys::console::log_1(&format!("result: {:?}", result).into());
            let result = invoke::<MyResult<(), String>>("set_settings", SetSettingsArg::new("num_top_ranked_to_try".into(), SettingsEnum::Usize(num_top_ranked_to_try.get_untracked()))).await;
            web_sys::console::log_1(&format!("result: {:?}", result).into());
            let result = invoke::<MyResult<(), String>>("set_settings", SetSettingsArg::new("sample_iterations".into(), SettingsEnum::Usize(sample_iterations.get_untracked()))).await;
            web_sys::console::log_1(&format!("result: {:?}", result).into());
            let result = invoke::<MyResult<(), String>>("set_settings", SetSettingsArg::new("update_probability_skip_stride".into(), SettingsEnum::Usize(update_probability_skip_stride.get_untracked()))).await;
            web_sys::console::log_1(&format!("result: {:?}", result).into());
            navigate("/pcb", Default::default());
        });        
    };

    Effect::new( move |_| {
        web_sys::console::log_1(&"SettingsPage initialized".into());
        if !initialized.get(){
            set_initialized.set(true);
            spawn_local(async move {
                // Here you can load settings from a file or database
                // For now, we just log that the settings page has been initialized
                // fetch all the settings from the backend
                let result: SettingsEnum = invoke("get_settings", GetSettingsArg::new("use_bayesian_inference".into())).await;
                set_use_bayesian_inference.set(result.as_bool().unwrap());
                let result: SettingsEnum = invoke("get_settings", GetSettingsArg::new("astar_max_expansions".into())).await;
                set_astar_max_expansions.set(result.as_usize().unwrap());
                let result: SettingsEnum = invoke("get_settings", GetSettingsArg::new("astar_stride".into())).await;
                set_astar_stride.set(result.as_float().unwrap());
                let result: SettingsEnum = invoke("get_settings", GetSettingsArg::new("trace_score_halved".into())).await;
                set_trace_score_halved.set(result.as_float().unwrap());
                let result: SettingsEnum = invoke("get_settings", GetSettingsArg::new("opportunity_cost_halved".into())).await;
                set_opportunity_cost_halved.set(result.as_float().unwrap());
                let result: SettingsEnum = invoke("get_settings", GetSettingsArg::new("max_trace_generation_attempts".into())).await;
                set_max_trace_generation_attempts.set(result.as_usize().unwrap());
                let result: SettingsEnum = invoke("get_settings", GetSettingsArg::new("first_iteration_prior_probability".into())).await;
                set_first_iteration_prior_probability.set(result.as_float().unwrap());
                let result: SettingsEnum = invoke("get_settings", GetSettingsArg::new("second_iteration_prior_probability".into())).await;
                set_second_iteration_prior_probability.set(result.as_float().unwrap());
                let result: SettingsEnum = invoke("get_settings", GetSettingsArg::new("second_iteration_num_traces".into())).await;
                set_second_iteration_num_traces.set(result.as_usize().unwrap());
                let result: SettingsEnum = invoke("get_settings", GetSettingsArg::new("via_cost".into())).await;
                set_via_cost.set(result.as_float().unwrap());
                let result: SettingsEnum = invoke("get_settings", GetSettingsArg::new("num_top_ranked_to_try".into())).await;
                set_num_top_ranked_to_try.set(result.as_usize().unwrap());
                let result: SettingsEnum = invoke("get_settings", GetSettingsArg::new("sample_iterations".into())).await;
                set_sample_iterations.set(result.as_usize().unwrap());
                let result: SettingsEnum = invoke("get_settings", GetSettingsArg::new("update_probability_skip_stride".into())).await;
                set_update_probability_skip_stride.set(result.as_usize().unwrap());
            });
        }
    });
    view! {
        <div class="p-8 max-w-4xl mx-auto">
            <h1 class="text-3xl font-bold mb-6">Settings</h1>
            <button
                on:click=on_back_clicked
                class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded mb-4"
            >
                "Back to PCB"
            </button>
            <form
                on:submit=move |ev| {
                    ev.prevent_default();
                    log::info!("Form submitted");
                }
                class="space-y-8"
            >
                // General Section
                <div>
                    <h2 class="text-xl font-semibold mb-4">General</h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <label class="flex items-center gap-2">
                            <input
                                checked=use_bayesian_inference
                                on:change=move |ev| {
                                    let input = event_target_checked(&ev);
                                    set_use_bayesian_inference.set(input);
                                }
                                type="checkbox"
                                class="form-checkbox"
                            />
                            "Use Bayesian Inference"
                        </label>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">
                                "A* Max Expansions"
                            </label>
                            <label class="block text-sm text-gray-500">"Recommended: 3000"</label>
                            <input
                                value=astar_max_expansions
                                on:change=move |ev| {
                                    let input = event_target_value(&ev);
                                    set_astar_max_expansions.set(input.parse().unwrap_or(0));
                                }
                                type="number"
                                min="1"
                                class="mt-1 block w-full rounded border-gray-300 shadow-sm"
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">
                                "A* Stride"
                            </label>
                            <label class="block text-sm text-gray-500">"Recommended: 1.27"</label>
                            <input
                                value=astar_stride
                                on:change=move |ev| {
                                    let input = event_target_value(&ev);
                                    set_astar_stride.set(input.parse().unwrap_or(0.0));
                                }
                                type="number"
                                step="0.01"
                                min="0.01"
                                class="mt-1 block w-full rounded border-gray-300 shadow-sm"
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">
                                "Trace Score Prob. Halved"
                            </label>
                            <label class="block text-sm text-gray-500">"Recommended: 10"</label>
                            <input
                                value=trace_score_halved
                                on:change=move |ev| {
                                    let input = event_target_value(&ev);
                                    set_trace_score_halved.set(input.parse().unwrap_or(0.0));
                                }
                                type="number"
                                step="0.01"
                                min="0.1"
                                class="mt-1 block w-full rounded border-gray-300 shadow-sm"
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">
                                "Opportunity Cost Prob. Halved"
                            </label>
                            <label class="block text-sm text-gray-500">"Recommended: 0.5"</label>
                            <input
                                value=opportunity_cost_halved
                                on:change=move |ev| {
                                    let input = event_target_value(&ev);
                                    set_opportunity_cost_halved.set(input.parse().unwrap_or(0.0));
                                }
                                type="number"
                                step="0.01"
                                min="0.1"
                                class="mt-1 block w-full rounded border-gray-300 shadow-sm"
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">
                                "Max Trace Generation Attempts"
                            </label>
                            <label class="block text-sm text-gray-500">"Recommended: 4"</label>
                            <input
                                value=max_trace_generation_attempts
                                on:change=move |ev| {
                                    let input = event_target_value(&ev);
                                    set_max_trace_generation_attempts
                                        .set(input.parse().unwrap_or(0));
                                }
                                type="number"
                                min="1"
                                class="mt-1 block w-full rounded border-gray-300 shadow-sm"
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">
                                "First Iteration Prior Probability"
                            </label>
                            <label class="block text-sm text-gray-500">"Recommended: 0.5"</label>
                            <input
                                value=first_iteration_prior_probability
                                on:change=move |ev| {
                                    let input = event_target_value(&ev);
                                    set_first_iteration_prior_probability
                                        .set(input.parse().unwrap_or(0.0));
                                }
                                type="number"
                                min="0.01"
                                max="1"
                                step="0.01"
                                class="mt-1 block w-full rounded border-gray-300 shadow-sm"
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">
                                "Second Iteration Prior Probability"
                            </label>
                            <label class="block text-sm text-gray-500">"Recommended: 0.4"</label>
                            <input
                                value=second_iteration_prior_probability
                                on:change=move |ev| {
                                    let input = event_target_value(&ev);
                                    set_second_iteration_prior_probability
                                        .set(input.parse().unwrap_or(0.0));
                                }
                                type="number"
                                min="0.01"
                                max="1"
                                step="0.01"
                                class="mt-1 block w-full rounded border-gray-300 shadow-sm"
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">
                                "Second Iteration Num Traces"
                            </label>
                            <label class="block text-sm text-gray-500">"Recommended: 3"</label>
                            <input
                                value=second_iteration_num_traces
                                on:change=move |ev| {
                                    let input = event_target_value(&ev);
                                    set_second_iteration_num_traces.set(input.parse().unwrap_or(0));
                                }
                                type="number"
                                min="1"
                                class="mt-1 block w-full rounded border-gray-300 shadow-sm"
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">
                                "Via Cost (mm)"
                            </label>
                            <label class="block text-sm text-gray-500">"Recommended: 5.0"</label>
                            <input
                                value=via_cost
                                on:change=move |ev| {
                                    let input = event_target_value(&ev);
                                    set_via_cost.set(input.parse().unwrap_or(0.0));
                                }
                                type="number"
                                min="0.0"
                                step="0.01"
                                class="mt-1 block w-full rounded border-gray-300 shadow-sm"
                            />
                        </div>
                    </div>
                </div>

                // Bayesian Section
                <div>
                    <h2 class="text-xl font-semibold mb-4">Bayesian Inference Options</h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700">
                                "Num Top Ranked to Try"
                            </label>
                            <label class="block text-sm text-gray-500">"Recommended: 3"</label>
                            <input
                                value=num_top_ranked_to_try
                                on:change=move |ev| {
                                    let input = event_target_value(&ev);
                                    set_num_top_ranked_to_try.set(input.parse().unwrap_or(0));
                                }
                                type="number"
                                min="1"
                                class="mt-1 block w-full rounded border-gray-300 shadow-sm"
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">
                                "Sample Iterations (1 or 2)"
                            </label>
                            <label class="block text-sm text-gray-500">"Recommended: 2"</label>
                            <input
                                value=sample_iterations
                                on:change=move |ev| {
                                    let input = event_target_value(&ev);
                                    set_sample_iterations.set(input.parse().unwrap_or(0));
                                }
                                type="number"
                                min="1"
                                max="2"
                                class="mt-1 block w-full rounded border-gray-300 shadow-sm"
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">
                                "Update Prob. Skip Stride"
                            </label>
                            <label class="block text-sm text-gray-500">"Recommended: 2"</label>
                            <input
                                value=update_probability_skip_stride
                                on:change=move |ev| {
                                    let input = event_target_value(&ev);
                                    set_update_probability_skip_stride
                                        .set(input.parse().unwrap_or(0));
                                }
                                type="number"
                                min="1"
                                class="mt-1 block w-full rounded border-gray-300 shadow-sm"
                            />
                        </div>
                    </div>
                </div>
            // Submit Button
            // <div>
            // <button type="submit" class="bg-blue-600 text-white px-6 py-2 rounded hover:bg-blue-700 shadow">
            // "Save Settings"
            // </button>
            // </div>
            </form>
        </div>
    }
}