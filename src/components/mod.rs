use leptos::prelude::*;

pub mod backdrop;
pub mod button;
pub mod canvas;
pub mod controls;
pub mod input;
pub mod loading;
pub mod paste_overlay;
pub mod pattern_card;
pub mod pattern_library;
pub mod popover;
pub mod selection_layer;
pub mod selection_menu;
pub mod selection_overlay;
pub mod sidebar;
pub mod status;
pub mod surface;
pub mod text;
pub mod toast;
pub mod tooltip;

pub use crate::components::backdrop::*;
pub use crate::components::button::*;
pub use crate::components::canvas::*;
pub use crate::components::controls::*;
pub use crate::components::input::*;
pub use crate::components::loading::*;
pub use crate::components::paste_overlay::*;
pub use crate::components::pattern_card::*;
pub use crate::components::pattern_library::*;
pub use crate::components::popover::*;
pub use crate::components::selection_layer::*;
pub use crate::components::selection_menu::*;
pub use crate::components::selection_overlay::*;
pub use crate::components::sidebar::*;
pub use crate::components::status::*;
pub use crate::components::surface::*;
pub use crate::components::text::*;
pub use crate::components::toast::*;
pub use crate::components::tooltip::*;

#[component]
pub fn Divider() -> impl IntoView {
    view! { <div class="border-l border-neutral-700 self-stretch my-2"></div> }
}

#[component]
pub fn Icon(icon: icondata::Icon) -> impl IntoView {
    view! { <leptos_icons::Icon icon=icon attr:class="w-6 h-6"></leptos_icons::Icon> }
}
