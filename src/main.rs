use gol::{app::App, components::LoadingCanvasProvider};
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

fn main() {
    console_error_panic_hook::set_once();

    let document = web_sys::window().unwrap().document().unwrap();
    if let Some(spinner) = document.get_element_by_id("loading-container") {
        spinner.remove();
    }

    mount_to_body(|| {
        view! {
            <LoadingCanvasProvider>
                <Router>
                    <Routes fallback=|| "Not found.">
                        <Route path=path!("/") view=App />
                        <Route path=path!("/:name") view=App />
                    </Routes>
                </Router>
            </LoadingCanvasProvider>
        }
    });
}
