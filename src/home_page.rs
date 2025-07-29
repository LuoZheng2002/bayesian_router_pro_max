use leptos::{prelude::*, reactive::spawn_local};
use leptos_router::hooks::use_navigate;
use shared::my_result::MyResult;
use tauri_sys::core::invoke;

// use crate::{nav_bar::NavBar};

#[component]
pub fn HomePage() -> impl IntoView {
    let (hint_message, set_hint_message) = signal("This is a hint message".to_string());
    let on_open_file_click = move |_| {
        spawn_local(async move {
            // This will invoke the Tauri command to open a file dialog
            let result: MyResult<(), String> = invoke("open_file", ()).await;
            match result {
                MyResult::Ok(_) => {
                    // Handle success if needed
                }
                MyResult::Err(err) => {
                    web_sys::console::error_1(&format!("Error opening file: {}", err).into());
                    set_hint_message.set(format!("Error opening file: {}", err));
                }
            }
        });
    };
    view! {
        <div class="min-h-screen bg-gray-50 flex flex-col items-center justify-center p-8 space-y-6">
            <div class="text-center space-y-2">
                <h1 class="text-4xl font-extrabold text-blue-700">"Bayesian Router Pro Max"</h1>
                <p class="text-base text-gray-400">"An automatic PCB routing tool using discrete Bayesian inference"</p>
                <p class="text-lg text-gray-700">"UM-SJTU Joint Institute MDE Group Project"</p>
            </div>

            <div class="bg-white shadow-md rounded-xl p-6 w-full max-w-md space-y-4 text-center">
                <h2 class="text-xl font-semibold text-gray-800">"Team Members"</h2>
                <div class="space-y-1 text-gray-600">
                    <p>"Zheng Luo"</p>
                    <p>"Xiaomi Zhou"</p>
                    <p>"Run Gan"</p>
                    <p>"Qixuan Chen"</p>
                </div>
                <p class="mt-4 text-gray-600">
                    <span class="font-medium text-gray-800">"Instructor: "</span> "Prof. An Zou"
                </p>
            </div>

            <div class="flex flex-col items-center space-y-3">
                <button
                    class="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-6 rounded-lg shadow-md transition"
                    on:click=on_open_file_click
                >
                    "Open File"
                </button>
                <p class="text-sm text-gray-500 italic">"Supported file format: Specctra DSN (.dsn)"</p>
                <div class="text-red-600 text-base font-medium text-center">{hint_message}</div>
            </div>
        </div>
    }
}