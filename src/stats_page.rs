use leptos::{component, prelude::{signal, Effect}, reactive::spawn_local, view, IntoView};
use leptos::prelude::*;


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
            spawn_local(async move{
                set_total_length.set(1234.56);
                set_num_vias.set(42);
                set_time_elapsed.set(12.34);
                set_num_bayesian_path_finding_calls.set(5);
                set_num_naive_path_finding_calls.set(10);
            });
            // Here you would typically fetch the statistics from your application state or context
            // For demonstration, we will just set some dummy values            
        }        
    });
    view! {
        <div class="stats-page">
            <h1>"Statistics"</h1>
            <p>"Here you can view various statistics."</p>
        </div>
    }
}