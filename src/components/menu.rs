use leptos::portal::Portal;
use leptos::prelude::*;

use crate::components::Button;

#[derive(Clone)]
pub struct MenuContext {
    pub open: ReadSignal<bool, LocalStorage>,
    pub set_open: WriteSignal<bool, LocalStorage>,
}

#[component]
pub fn Menu(children: ChildrenFn) -> impl IntoView {
    let (open, set_open) = signal_local(false);
    provide_context(MenuContext { open, set_open });

    view! {
        <Portal mount=document().body().unwrap()>
            <div class=move || {
                format!(
                    "z-50 fixed inset-y-0 right-0 bg-neutral-900 transition-transform transition-300 {}",
                    if open.get() { "translate-x-0" } else { "translate-x-full" },
                )
            }>{children()}</div>

        </Portal>
    }
}

#[component]
pub fn MenuTrigger(children: Children) -> impl IntoView {
    let MenuContext { set_open, .. } = use_context::<MenuContext>().unwrap();
    view! {
        <div class="absolute top-16 left-0 -translate-x-full [writing-mode:sideways-lr]">
            <Button
                class="rounded-l-md px-1"
                on_press=move || {
                    set_open.update(|b| *b = !*b);
                }
            >

                {children()}
            </Button>
        </div>
    }
}
