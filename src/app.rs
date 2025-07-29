use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use leptos_router::hooks::use_navigate;
use leptos_router::{components::{ParentRoute, Route, Router, Routes}, path};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;
use shared::pcb_render_model::PcbRenderModel;
use wasm_bindgen::prelude::*;

use crate::app_state::AppState;
use crate::home_page::HomePage;
use crate::pcb_page::PcbPage;
use crate::settings_page::SettingsPage;
use tauri_sys::{event};
use futures::StreamExt;

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}


#[component]
pub fn App() -> impl IntoView {
  let navigate = use_navigate();
  let navigate2 = navigate.clone();

  provide_context(AppState::new());

  let app_state = use_context::<AppState>().expect("AppState context not found");

    // Listen for events
    spawn_local(async move {
        // let _unlisten = event::listen::<String>("backend-to-frontend-event", |event: Event<String>| {
        //     log::info!("Received event: {}", event.payload());
        // }).await;
        let navigate_to_event_name = "string-event";
        let mut events = event::listen::<(String, String)>(navigate_to_event_name).await.unwrap();
        while let Some(event) = events.next().await {
          assert_eq!(event.event, navigate_to_event_name);
          web_sys::console::log_1(&format!("Got a message: {}, {}, {}", event.event, event.payload.0, event.payload.1).into());
          match (event.payload.0.as_str(), event.payload.1.as_str()) {
            ("navigate", "home") =>{
              web_sys::console::log_1(&"Navigating to Home page".into());
              navigate("/", Default::default());
            },
            ("navigate", "pcb") =>{
              web_sys::console::log_1(&"Navigating to PCB page".into());
              navigate("/pcb", Default::default());
            },
            ("navigate", "settings") =>{
              web_sys::console::log_1(&"Navigating to Settings page".into());
              navigate("/settings", Default::default());
            },
            ("navigate", "stats") =>{
              web_sys::console::log_1(&"Navigating to View Stats page".into());
              navigate("/stats", Default::default());
            },
            ("enable", "step-in") =>{
              web_sys::console::log_1(&"Stepping in".into());
              app_state.step_in_disabled.set(false);
            },
            ("enable", "step-out") =>{
              web_sys::console::log_1(&"Stepping out".into());
              app_state.step_out_disabled.set(false);
            },
            ("enable", "step-over") =>{
              web_sys::console::log_1(&"Stepping over".into());
              app_state.step_over_disabled.set(false);
            },
            ("enable", "view-stats") =>{
              web_sys::console::log_1(&"Viewing stats".into());
              app_state.view_stats_disabled.set(false);
            },            
            ("enable", "save-result") =>{
              web_sys::console::log_1(&"Saving result".into());
              app_state.save_result_disabled.set(false);
            },
            ("disable", "step-in") =>{
              web_sys::console::log_1(&"Stepping in".into());
              app_state.step_in_disabled.set(true);
            },
            ("disable", "step-out") =>{
              web_sys::console::log_1(&"Stepping out".into());
              app_state.step_out_disabled.set(true);
            },
            ("disable", "step-over") =>{
              web_sys::console::log_1(&"Stepping over".into());
              app_state.step_over_disabled.set(true);
            },
            ("disable", "view-stats") =>{
              web_sys::console::log_1(&"Viewing stats".into());
              app_state.view_stats_disabled.set(true);
            },            
            ("disable", "save-result") =>{
              web_sys::console::log_1(&"Saving result".into());
              app_state.save_result_disabled.set(true);
            },
            ("start-pause", "start") =>{
              web_sys::console::log_1(&"Starting algorithm".into());
              app_state.start_pause_str.set("Pause".into());
              println!("Setting start_pause_str to Pause");
            },
            ("start-pause", "pause") =>{
              web_sys::console::log_1(&"Pausing algorithm".into());
              app_state.start_pause_str.set("Start".into());
              println!("Setting start_pause_str to Start");
            },
            ("hint-message", hint_message) =>{
              web_sys::console::log_1(&format!("Setting hint message: {}", hint_message).into());
              app_state.hint_message.set(hint_message.to_string());
            },
            _=>{
              web_sys::console::log_1(&format!("Unknown navigation event: {}, {}", event.payload.0, event.payload.1).into());
            }
          }            
        }
    });
    spawn_local(async move {
      let submit_render_model_name = "submit-pcb-render-model";
        let mut events = event::listen::<PcbRenderModel>(submit_render_model_name).await.unwrap();
        while let Some(event) = events.next().await {
          assert_eq!(event.event, submit_render_model_name);
          // web_sys::console::log_1(&format!("Got a message: {}, {}", event.event, event.payload).into());
          let mut pcb_render_model = app_state.pcb_render_model.write();
          let mut render_next_frame = app_state.render_next_frame.write();
          *pcb_render_model = Some(event.payload);
          *render_next_frame = true;
          web_sys::console::log_1(&format!("Received PCB render model").into());
        }
      }
    );


    view! {
        <div id="root">
            <main>
                // <Routes/> both defines our routes and shows them on the page
                <Routes fallback=|| "Not found.">
                    // users like /gbj or /bob
                    <Route path=path!("/") view=HomePage />
                    <Route path=path!("/pcb") view=PcbPage />
                    <Route path=path!("/settings") view=SettingsPage />
                    <Route path=path!("/stats") view=crate::stats_page::StatsPage />
                    // a fallback if the /:id segment is missing from the URL
                    <Route
                        path=path!("")
                        view=move || view! { <p class="contact">"Select a contact."</p> }
                    />
                </Routes>
            </main>
        </div>
    }
}
