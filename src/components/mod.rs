use leptos::*;

pub mod button;
pub mod canvas;
pub mod controls;
pub mod status;

pub use crate::components::button::Button;
pub use crate::components::canvas::Canvas;
pub use crate::components::controls::Controls;
pub use crate::components::status::Status;

#[component]
pub fn Divider() -> impl IntoView {
    view! { <div class="border-l-2 border-white/20"></div> }
}

#[component]
pub fn Icon(icon: icondata::Icon) -> impl IntoView {
    view! { <leptos_icons::Icon icon=icon class="w-6 h-6 fill-current"></leptos_icons::Icon> }
}
