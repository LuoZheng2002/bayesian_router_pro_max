use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

// use crate::{nav_bar::NavBar};

#[component]
pub fn HomePage() -> impl IntoView {
    let navigate = use_navigate();
    let on_pcb_click = move |_| {
        navigate("/pcb", Default::default());
    };
    view! {
        <div>
            <h1>"Welcome to the Home Page"</h1>
            <p>"This is the home page of our application."</p>
            <button on:click=on_pcb_click>"Go to PCB Page"</button>
        </div>
    }
}