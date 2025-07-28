use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

// use bayesian inference
// max_trials
// astar stride
// half probability raw score
// half probability opportunity cost
// max generation attempts
// first iteration prior probability
// second iteration prior probability
// second iteration num traces
// via cost (mm)

// bayesian inference related:
// num top ranked to try
// sample iterations (1 or 2)
// update proba skip stride

#[component]
pub fn SettingsPage() -> impl IntoView {
    let navigate = use_navigate();

    let on_back_clicked = move |_|{
        navigate("/pcb", Default::default());
    };
    view! {
        <div class="p-4">
            <h1 class="text-2xl font-bold mb-4">"Settings"</h1>
            <button on:click=on_back_clicked class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded mb-4" >
                "Back to PCB"
            </button>
            <form class="mt-4">
                <label class="block mb-2">
                    "Setting 1:"
                    <input type="text" class="border p-2 w-full" placeholder="Enter setting 1"/>
                </label>
                <label class="block mb-2">
                    "Setting 2:"
                    <input type="text" class="border p-2 w-full" placeholder="Enter setting 2"/>
                </label>
                <button type="submit" class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
                    "Save Settings"
                </button>
            </form>
        </div>
    }
}