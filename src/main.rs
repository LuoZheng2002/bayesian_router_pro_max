use leptos_router::components::Router;
use test_desktop_ui::app::App;


use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <Router>
                <App />
            </Router>
        }
    })
}
