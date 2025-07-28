use leptos::{prelude::*, reactive::spawn_local};
use leptos_router::hooks::use_navigate;
use serde_wasm_bindgen::to_value;
use shared::settings_enum::{GetSettingsArg, SettingsEnum};
use tauri_sys::core::invoke;

// use bayesian inference    bool
// astar max expansions     usize >=1 recommended 1000     
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

    let on_back_clicked = move |_|{

        navigate("/pcb", Default::default());
    };
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
            <button on:click=on_back_clicked class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded mb-4" >
                "Back to PCB"
            </button>
            <form on:submit=move |ev| {
            ev.prevent_default(); // prevent full-page reload
            log::info!("Form submitted");
        } class="space-y-8">
                // General Section
                <div>
                    <h2 class="text-xl font-semibold mb-4">General</h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <label class="flex items-center gap-2">
                            <input checked=use_bayesian_inference type="checkbox" class="form-checkbox" />
                            "Use Bayesian Inference"
                        </label>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">"A* Max Expansions"</label>
                            <label class="block text-sm text-gray-500">"Recommended: 1000"</label>
                            <input value=astar_max_expansions type="number" min="1" class="mt-1 block w-full rounded border-gray-300 shadow-sm" />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">"A* Stride"</label>
                            <label class="block text-sm text-gray-500">"Recommended: 1.27"</label>
                            <input value=astar_stride type="number" step="0.01" min="0.01" class="mt-1 block w-full rounded border-gray-300 shadow-sm" />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">"Trace Score Prob. Halved"</label>
                            <label class="block text-sm text-gray-500">"Recommended: 10"</label>
                            <input value=trace_score_halved type="number" step="0.01" min="0.1" class="mt-1 block w-full rounded border-gray-300 shadow-sm" />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">"Opportunity Cost Prob. Halved"</label>
                            <label class="block text-sm text-gray-500">"Recommended: 0.5"</label>
                            <input value=opportunity_cost_halved type="number" step="0.01" min="0.1" class="mt-1 block w-full rounded border-gray-300 shadow-sm" />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">"Max Trace Generation Attempts"</label>
                            <label class="block text-sm text-gray-500">"Recommended: 4"</label>
                            <input value=max_trace_generation_attempts type="number" min="1" class="mt-1 block w-full rounded border-gray-300 shadow-sm" />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">"First Iteration Prior Probability"</label>
                            <label class="block text-sm text-gray-500">"Recommended: 0.5"</label>
                            <input value=first_iteration_prior_probability type="number" min="0.01" max="1" step="0.01" class="mt-1 block w-full rounded border-gray-300 shadow-sm" />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">"Second Iteration Prior Probability"</label>
                            <label class="block text-sm text-gray-500">"Recommended: 0.4"</label>
                            <input value=second_iteration_prior_probability type="number" min="0.01" max="1" step="0.01" class="mt-1 block w-full rounded border-gray-300 shadow-sm" />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">"Second Iteration Num Traces"</label>
                            <label class="block text-sm text-gray-500">"Recommended: 3"</label>
                            <input value=second_iteration_num_traces type="number" min="1" class="mt-1 block w-full rounded border-gray-300 shadow-sm" />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">"Via Cost (mm)"</label>
                            <label class="block text-sm text-gray-500">"Recommended: 5.0"</label>
                            <input value=via_cost type="number" min="0.0" step="0.01" class="mt-1 block w-full rounded border-gray-300 shadow-sm" />
                        </div>
                    </div>
                </div>

                // Bayesian Section
                <div>
                    <h2 class="text-xl font-semibold mb-4">Bayesian Inference Options</h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700">"Num Top Ranked to Try"</label>
                            <label class="block text-sm text-gray-500">"Recommended: 3"</label>
                            <input value=num_top_ranked_to_try type="number" min="1" class="mt-1 block w-full rounded border-gray-300 shadow-sm" />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">"Sample Iterations (1 or 2)"</label>
                            <label class="block text-sm text-gray-500">"Recommended: 2"</label>
                            <input value=sample_iterations type="number" min="1" max="2" class="mt-1 block w-full rounded border-gray-300 shadow-sm" />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700">"Update Prob. Skip Stride"</label>
                            <label class="block text-sm text-gray-500">"Recommended: 2"</label>
                            <input value=update_probability_skip_stride type="number" min="1" class="mt-1 block w-full rounded border-gray-300 shadow-sm" />
                        </div>
                    </div>
                </div>
                // Submit Button
                // <div>
                //     <button type="submit" class="bg-blue-600 text-white px-6 py-2 rounded hover:bg-blue-700 shadow">
                //         "Save Settings"
                //     </button>
                // </div>
            </form>
        </div>        
    }
}