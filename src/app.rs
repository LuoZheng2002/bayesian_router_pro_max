use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use leptos_router::hooks::use_navigate;
use leptos_router::{components::{ParentRoute, Route, Router, Routes}, path};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;

use crate::app_state::AppState;
use crate::home_page::HomePage;
use crate::pcb_page::PcbPage;
use crate::settings_page::SettingsPage;
use tauri_sys::{event};
use futures::StreamExt;

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
//     pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;
// }

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}


#[component]
pub fn App() -> impl IntoView {
  let navigate = use_navigate();

  provide_context(AppState::new());

  let (message, set_message) = signal("Hello world".to_string());

    // Listen for events
    spawn_local(async move {
        // let _unlisten = event::listen::<String>("backend-to-frontend-event", |event: Event<String>| {
        //     log::info!("Received event: {}", event.payload());
        // }).await;
        let navigate_to_event_name = "navigate-to";
        let mut events = event::listen::<String>(navigate_to_event_name).await.unwrap();
        while let Some(event) = events.next().await {
          assert_eq!(event.event, navigate_to_event_name);
          web_sys::console::log_1(&format!("Got a message: {}, {}", event.event, event.payload).into());
          match event.payload.as_str(){
            "pcb" =>{
              web_sys::console::log_1(&"Navigating to PCB page".into());
              navigate("/pcb", Default::default());
            },
            "settings" =>{
              web_sys::console::log_1(&"Navigating to Settings page".into());
              navigate("/settings", Default::default());
            },
            _=>{
              web_sys::console::log_1(&format!("Unknown navigation event: {}", event.payload).into());
            }
          }            
        }
    });


    view! {    
    <div id="root">
      <div>{message}</div>
      // we wrap the whole app in a <Router/> to allow client-side navigation
      // from our nav links below      
      // <Router>
        <main>
          // <Routes/> both defines our routes and shows them on the page
          <Routes fallback=|| "Not found.">
              // users like /gbj or /bob
              <Route
                path=path!("/")
                view=HomePage
              />
              <Route
                path=path!("/pcb")
                view=PcbPage
              />
              <Route
                path=path!("/settings")
                view=SettingsPage
              />
              // a fallback if the /:id segment is missing from the URL
              <Route
                path=path!("")
                view=move || view! { <p class="contact">"Select a contact."</p> }
              />
          </Routes>
        </main>
      // </Router>
    </div>
  }
}
