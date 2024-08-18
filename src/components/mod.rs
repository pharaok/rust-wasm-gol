use leptos::*;

pub mod button;
pub mod canvas;
pub mod controls;
pub mod loading;
pub mod menu;
pub mod pattern_card;
pub mod pattern_library;
pub mod status;
pub mod text;

pub use crate::components::button::*;
pub use crate::components::canvas::*;
pub use crate::components::controls::*;
pub use crate::components::loading::*;
pub use crate::components::menu::*;
pub use crate::components::pattern_card::*;
pub use crate::components::pattern_library::*;
pub use crate::components::status::*;
pub use crate::components::text::*;

#[component]
pub fn Divider() -> impl IntoView {
    view! { <div class="border-l-2 border-neutral-700"></div> }
}

#[component]
pub fn Icon(icon: icondata::Icon) -> impl IntoView {
    view! { <leptos_icons::Icon icon=icon class="w-6 h-6 fill-current"></leptos_icons::Icon> }
}
