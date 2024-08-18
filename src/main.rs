#![feature(cell_update)]
use gol::{app::App, components::create_loading_canvas};
use leptos::*;
use leptos_router::*;

fn main() {
    console_error_panic_hook::set_once();

    provide_context(create_loading_canvas());

    mount_to_body(|| {
        view! {
            <Router>
                <Routes>
                    <Route path="/" view=App/>
                    <Route path="/:name" view=App/>
                </Routes>
            </Router>
        }
    });
}
