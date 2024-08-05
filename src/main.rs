#![feature(cell_update)]
use leptos::*;
use leptos_router::*;

use crate::app::App;

mod app;
mod components;
mod draw;
mod parse;
mod quadtree;
mod universe;

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
