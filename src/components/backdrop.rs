use leptos::attr::any_attribute::AnyAttribute;
use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Backdrop(
    children: Children,
    #[prop(into, optional)] class: TextProp,
    #[prop(attrs)] attrs: Vec<AnyAttribute>,
) -> impl IntoView {
    view! {
        <div
            class=move || {
                tw_merge!("bg-neutral-950 rounded-xl",
                    class.get().to_string())
            }
            {..attrs}
        >

            {children()}
        </div>
    }
}
