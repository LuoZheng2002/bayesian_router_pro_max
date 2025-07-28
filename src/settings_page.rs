use leptos::prelude::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <div class="p-4">
            <h1 class="text-2xl font-bold mb-4">"Settings"</h1>
            <p>"This is the settings page where you can configure your application."</p>
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