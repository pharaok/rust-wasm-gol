use leptos::*;

#[component]
pub fn Button<F>(
    children: Children,
    on_press: F,
    #[prop(into, default = MaybeSignal::Static(false))] disabled: MaybeSignal<bool>,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    view! {
        <button
            class=Signal::derive(move || {
                format!(
                    "rounded-md p-2 bg-gray-800 transition {}",
                    if disabled() { "text-gray-500" } else { "text-white" },
                )
            })

            disabled=disabled
            on:click=move |_| {
                on_press();
            }
        >

            {children()}
        </button>
    }
}
