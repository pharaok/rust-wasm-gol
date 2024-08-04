use leptos::*;

pub enum ButtonVariant {
    Standard,
    Icon,
}

#[component]
pub fn Button<F>(
    children: Children,
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
                format!(
                    "transition text-white disabled:text-gray-500 enabled:hover:bg-white/10 {}",
                    match variant {
                        ButtonVariant::Standard => "rounded-md p-2",
                        ButtonVariant::Icon => "flex justify-center items-center p-2",
                    },
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
