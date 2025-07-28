use leptos::{component, prelude::{signal, Effect}, reactive::spawn_local, view, IntoView};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use shared::stats_enum::{StatsArgs, StatsEnum};
use tauri_sys::core::invoke;


#[component]
pub fn StatsPage() -> impl IntoView {
    let (total_length, set_total_length) = signal::<f64>(0.0);
    let (num_vias, set_num_vias) = signal::<usize>(0);
    let (time_elapsed, set_time_elapsed) = signal::<f64>(0.0);
    let (num_bayesian_path_finding_calls, set_num_bayesian_path_finding_calls) = signal::<usize>(0);
    let (num_naive_path_finding_calls, set_num_naive_path_finding_calls) = signal::<usize>(0);
    let (initialized, set_initialized) = signal::<bool>(false);
    Effect::new(move || {
        if !initialized.get() {
            set_initialized.set(true);
            println!("Initializing statistics...");
            spawn_local(async move{
                let result: StatsEnum = invoke("get_stats", StatsArgs::new("total_length".to_string())).await;
                set_total_length.set(result.as_float().unwrap_or(0.0));
                let result: StatsEnum = invoke("get_stats", StatsArgs::new("num_vias".to_string())).await;
                set_num_vias.set(result.as_usize().unwrap_or(0));
                let result: StatsEnum = invoke("get_stats", StatsArgs::new("time_elapsed".to_string())).await;
                set_time_elapsed.set(result.as_float().unwrap_or(0.0));
                let result: StatsEnum = invoke("get_stats", StatsArgs::new("num_bayesian_path_finding_calls".to_string())).await;
                set_num_bayesian_path_finding_calls.set(result.as_usize().unwrap_or(0));
                let result: StatsEnum = invoke("get_stats", StatsArgs::new("num_naive_path_finding_calls".to_string())).await;
                set_num_naive_path_finding_calls.set(result.as_usize().unwrap_or(0));
            });
            // Here you would typically fetch the statistics from your application state or context
            // For demonstration, we will just set some dummy values            
        }        
    });
    let navigate = use_navigate();
    let on_back_clicked = move |_|{
        navigate("/pcb", Default::default());
    };
    view! {
        <div class="p-6 space-y-6">
            <h1 class="text-2xl font-bold text-center">"Statistics"</h1>

            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                <StatCardF64 label="Total Length (mm)" value=total_length />
                <StatCardUsize label="Number of Vias" value=num_vias />
                <StatCardF64 label="Time Elapsed (s)" value=time_elapsed />
                <StatCardUsize label="Bayesian Pathfinding Calls" value=num_bayesian_path_finding_calls />
                <StatCardUsize label="Naive Pathfinding Calls" value=num_naive_path_finding_calls />
            </div>
            <button
                on:click=on_back_clicked
                class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded mb-4"
            >
                "Back to PCB"
            </button>
        </div>
        
    }
}

#[component]
fn StatCardF64(label: &'static str, value: ReadSignal<f64>) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 shadow-md rounded-xl p-4">
            <div class="text-sm text-gray-500 dark:text-gray-400">{label}</div>
            <div class="text-xl font-semibold text-gray-900 dark:text-white">
                {move || value.get().to_string()}
            </div>
        </div>
    }
}

#[component]
fn StatCardUsize(label: &'static str, value: ReadSignal<usize>) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 shadow-md rounded-xl p-4">
            <div class="text-sm text-gray-500 dark:text-gray-400">{label}</div>
            <div class="text-xl font-semibold text-gray-900 dark:text-white">
                {move || value.get().to_string()}
            </div>
        </div>
    }
}