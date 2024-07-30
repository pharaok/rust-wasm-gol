#![feature(cell_update)]
use leptos::*;

use crate::components::Canvas;

mod components;
mod draw;
mod quadtree;
mod universe;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! { <Canvas/> }
    });
}
