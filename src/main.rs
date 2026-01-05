use gol::{app::App, components::LoadingCanvasProvider};
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! {
            <LoadingCanvasProvider>
                <Router>
                    <Routes fallback=|| "Not found.">
                        <Route path=path!("/") view=|| view! { <App /> } />
                        <Route path=path!("/:name") view=|| view! { <App /> } />
                        <Route path=path!("/meta") view=|| view! { <Redirect path="/" /> } />
                        <Route path=path!("/meta/:name") view=|| view! { <App meta=true /> } />
                    </Routes>
                </Router>
            </LoadingCanvasProvider>
        }
    });
}
