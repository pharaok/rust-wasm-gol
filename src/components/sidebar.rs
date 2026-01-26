use leptos::portal::Portal;
use leptos::prelude::*;

use crate::components::{Backdrop, Button, Surface};

#[derive(Clone)]
pub struct MenuContext {
    pub open: ReadSignal<bool, LocalStorage>,
    pub set_open: WriteSignal<bool, LocalStorage>,
}

#[component]
pub fn Sidebar(children: ChildrenFn) -> impl IntoView {
    let (open, set_open) = signal_local(false);
    provide_context(MenuContext { open, set_open });

    let children = StoredValue::new(children);
    view! {
        <Portal mount=document().body().unwrap()>
            <Backdrop class=move || {
                format!(
                    "z-40 fixed inset-y-0 right-0 rounded-none transition-transform duration-150 {}",
                    if open.get() { "translate-x-0" } else { "translate-x-full" },
                )
            }>{children.read_value()()}</Backdrop>
        </Portal>
    }
}

#[component]
pub fn SidebarTrigger(children: Children) -> impl IntoView {
    let MenuContext { set_open, .. } = use_context::<MenuContext>().unwrap();
    view! {
        <Surface class="absolute top-16 left-0 -translate-x-full [writing-mode:sideways-lr] rounded-r-none">
            <Button
                class="px-2 py-1 rounded-l-md"
                on_press=move || {
                    set_open.update(|b| *b = !*b);
                }
            >

                {children()}
            </Button>
        </Surface>
    }
}
