#![feature(hash_raw_entry)]

use canvas::Canvas;
use leptos::*;

mod button;
mod canvas;
mod hashlife;
mod icons;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| view! { <Canvas/> })
}
