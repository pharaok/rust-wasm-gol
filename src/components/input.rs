use leptos::attr::any_attribute::AnyAttribute;
use leptos::{html, prelude::*};
use tailwind_fuse::tw_merge;
use web_sys::File;

use crate::components::{Button, Divider, Surface};

#[component]
pub fn Input(
    #[prop(into, optional)] class: TextProp,
    #[prop(attrs)] attrs: Vec<AnyAttribute>,
) -> impl IntoView {
    view! {
        <input
            class=move || {
                tw_merge!(
                    "px-2 rounded-md bg-neutral-900",
                    class.get().to_string()
                )
            }
            {..attrs}
        />
    }
}

#[component]
pub fn TextArea(
    #[prop(into, optional)] class: TextProp,
    #[prop(attrs)] attrs: Vec<AnyAttribute>,
) -> impl IntoView {
    view! {
        <textarea
            class=move || {
                tw_merge!(
                    "p-2 rounded-md bg-neutral-900",
                    class.get().to_string()
                )
            }
            {..attrs}
        />
    }
}

#[component]
pub fn FileInput(
    #[prop(into)] on_change: Callback<File>,
    #[prop(optional, into)] accept: String,
    #[prop(attrs)] attrs: Vec<AnyAttribute>,
) -> impl IntoView {
    let input_ref = NodeRef::<html::Input>::new();
    let (file_name, set_file_name) = signal("No file selected.".to_owned());
    view! {
        <Surface class="flex items-center">
            <input
                r#type="file"
                node_ref=input_ref
                on:change=move |ev| {
                    let target = event_target::<web_sys::HtmlInputElement>(&ev);
                    if let Some(file) = target.files().and_then(|files| files.get(0)) {
                        set_file_name(file.name());
                        on_change.run(file);
                    }
                }
                class="hidden"
                accept={accept}
                {..attrs}
            />
            <Button
                attr:r#type="button"
                on_press=move || {
                    input_ref.get().unwrap().click();
                }
                class="rounded-l-md px-4"
            >
                Browse...
            </Button>
            <Divider />
            <span class="mx-2">{file_name}</span>
        </Surface>
    }
}
