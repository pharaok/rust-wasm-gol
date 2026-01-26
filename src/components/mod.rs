use leptos::prelude::*;

pub mod app_menu;
pub mod backdrop;
pub mod button;
pub mod canvas;
pub mod controls;
pub mod dialog;
pub mod input;
pub mod loading;
pub mod paste_layer;
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

pub use crate::components::app_menu::*;
pub use crate::components::backdrop::*;
pub use crate::components::button::*;
pub use crate::components::canvas::*;
pub use crate::components::controls::*;
pub use crate::components::dialog::*;
pub use crate::components::input::*;
pub use crate::components::loading::*;
pub use crate::components::paste_layer::*;
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

#[derive(Clone, Copy)]
pub enum IconSize {
    Small,
    Regular,
}
#[component]
pub fn Icon(
    icon: icondata::Icon,
    #[prop(optional, default=IconSize::Regular)] size: IconSize,
) -> impl IntoView {
    view! {
        <leptos_icons::Icon
            icon=icon
            attr:class=match size {
                IconSize::Small => "size-4",
                IconSize::Regular => "size-6",
            }
        ></leptos_icons::Icon>
    }
}
