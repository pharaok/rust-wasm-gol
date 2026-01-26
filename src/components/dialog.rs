use crate::components::{Backdrop, PopoverContext};
use leptos::portal::Portal;
use leptos::prelude::*;

#[component]
pub fn Dialog(children: ChildrenFn) -> impl IntoView {
    let PopoverContext {
        is_open,
        set_is_open,
        ..
    } = use_context::<PopoverContext>().unwrap();

    let children = StoredValue::new(children);
    view! {
        <Portal mount=document().body().unwrap()>
            <Show when=move || is_open.get()>
                <div
                    class="fixed inset-0 flex items-center justify-center bg-black/80 z-50"
                    on:click=move |_| {
                        set_is_open.set(false);
                    }
                >
                    <Backdrop
                        class="p-8 z-50"
                        on:click=move |ev| {
                            ev.stop_propagation();
                        }
                    >
                        {children.read_value()()}
                    </Backdrop>
                </div>
            </Show>
        </Portal>
    }
}
