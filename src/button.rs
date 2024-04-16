use leptos::*;

#[component]
pub fn Button<F>(children: Children, on_press: F) -> impl IntoView
where
    F: Fn() + 'static,
{
    view! {
        <button
            class="rounded-md p-2 bg-gray-800 text-white"
            on:click=move |_| {
                on_press();
            }
        >

            {children()}
        </button>
    }
}
