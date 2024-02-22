use canvas::Canvas;
use leptos::*;

mod canvas;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| view! { <Canvas/> })
}
