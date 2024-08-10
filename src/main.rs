#![feature(cell_update)]
use gol::app::App;
use leptos::*;
use leptos_router::*;

fn main() {
    console_error_panic_hook::set_once();

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
