use leptos::*;
use leptos_router::*;
use tailwind_fuse::tw_merge;

pub enum ButtonVariant {
    Standard,
    Icon,
}

#[component]
pub fn Button<F>(
    children: Children,
    #[prop(into, optional)] class: Option<String>,
    #[prop(default=ButtonVariant::Standard)] variant: ButtonVariant,
    on_press: F,
    #[prop(into, default = MaybeSignal::Static(false))] disabled: MaybeSignal<bool>,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    view! {
        <button
            class=move || {
                tw_merge!(
                    "transition text-white disabled:text-neutral-500 bg-neutral-900 enabled:hover:bg-neutral-800",
                    match variant { ButtonVariant::Standard => "rounded-md p-2", ButtonVariant::Icon
                    => "flex justify-center items-center p-2", }, class.clone()
                )
            }

            disabled=disabled
            on:click=move |_| {
                on_press();
            }
        >

            {children()}
        </button>
    }
}

#[component]
pub fn Link(
    children: Children,
    href: String,
    #[prop(into, optional)] class: Option<String>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
    view! {
        <A class=tw_merge!("text-blue-500 hover:underline", class) href=href.clone() {..attrs}>
            {children()}
        </A>
    }
}
