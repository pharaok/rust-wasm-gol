use leptos::*;
use leptos_use::use_timeout_fn;

use crate::components::Button;

#[derive(Clone)]
pub struct MenuContext {
    pub open: ReadSignal<bool>,
    pub set_open: WriteSignal<bool>,
}

#[component]
pub fn Menu(
    // open: ReadSignal<bool>,
    // set_open: WriteSignal<bool>,
    children: ChildrenFn,
) -> impl IntoView {
    let (open, set_open) = create_signal(false);
    provide_context(MenuContext { open, set_open });

    view! {
        <Portal mount=document().body().unwrap()>
            <div class=move || {
                format!(
                    "fixed inset-y-0 right-0 bg-neutral-900 text-white transition-transform transition-300 {}",
                    if open() { "translate-x-0" } else { "translate-x-full" },
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
                class="font-mono rounded-r-none px-1"
                on_press=move || {
                    set_open.update(|b| *b = !*b);
                }
            >

                {children()}
            </Button>
        </div>
    }
}
