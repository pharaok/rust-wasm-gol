use leptos::attr::any_attribute::AnyAttribute;
use leptos::prelude::*;
use leptos_router::components::*;
use tailwind_fuse::tw_merge;

pub enum ButtonVariant {
    Primary,
    Secondary,
    Icon,
}

const ICON_CLASS: &str = "flex justify-center items-center p-2";

#[component]
pub fn Button(
    children: Children,
    #[prop(into, optional)] class: TextProp,
    #[prop(default=ButtonVariant::Secondary)] variant: ButtonVariant,
    #[prop(into, optional)] on_press: Option<Callback<()>>,
    #[prop(into, default = false.into())] disabled: Signal<bool, LocalStorage>,
    #[prop(attrs)] attrs: Vec<AnyAttribute>,
) -> impl IntoView {
    view! {
        <button
            class=move || {
                tw_merge!(
                    "transition disabled:text-neutral-500 bg-neutral-900 enabled:hover:bg-neutral-800",
                    match variant {
                        ButtonVariant::Primary => "px-4 py-2 font-bold bg-white disabled:text-neutral-700 text-black enabled:hover:bg-neutral-400",
                        ButtonVariant::Secondary => "px-4 py-2",
                        ButtonVariant::Icon => ICON_CLASS,
                    },
                    class.get().to_string()
                )
            }

            disabled=move || disabled.get()
            on:click=move |_| {
                if let Some(cb) = on_press {
                    cb.run(());
                }
            }
            {..attrs}
        >

            {children()}
        </button>
    }
}

pub enum LinkVariant {
    Inline,
    Icon,
}
#[component]
pub fn Link(
    children: Children,
    href: String,
    #[prop(into, optional)] class: Option<String>,
    #[prop(default=LinkVariant::Inline)] variant: LinkVariant,
    #[prop(attrs)] attrs: Vec<AnyAttribute>,
) -> impl IntoView {
    let icon_class = format!("bg-neutral-900 hover:bg-neutral-800 {}", ICON_CLASS);
    view! {
        <A
            href=href
            attr:class=tw_merge!(
                "transition",
                match variant {
                    LinkVariant::Inline=>"text-blue-500 hover:underline",
                    LinkVariant::Icon => &icon_class,
                },
                class
            )
            {..attrs}
        >
            {children()}
        </A>
    }
}
