#![feature(cell_update)]
use canvas::Canvas;
use leptos::*;

mod button;
mod canvas;
mod icons;
mod patterns;
mod quadtree;
mod universe;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! { <Canvas/> }
    });
}
