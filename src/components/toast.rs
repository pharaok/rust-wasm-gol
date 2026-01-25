use leptos::{logging, portal::Portal, prelude::*};

use crate::components::Surface;

#[derive(Clone, Debug)]
struct Toast {
    pub id: usize,
    pub message: String,
    is_closing: RwSignal<bool, LocalStorage>,
}

#[derive(Clone)]
pub struct ToastContext {
    pub push_toast: Callback<String>,
}

#[component]
pub fn ToastRegion(children: Children) -> impl IntoView {
    let (toasts, set_toasts) = signal_local::<Vec<Toast>>(Vec::new());
    let id = StoredValue::new_local(0);

    let push_toast = Callback::new(move |message: String| {
        let curr_id = id.get_value();
        set_toasts.update(|ts| {
            ts.push(Toast {
                id: curr_id,
                message: message.to_owned(),
                is_closing: RwSignal::new_local(false),
            });
        });

        set_timeout(
            move || {
                toasts.with(|ts| {
                    if let Some(toast) = ts.iter().find(|t| t.id == curr_id) {
                        toast.is_closing.set(true);
                    }
                });
            },
            std::time::Duration::from_millis(3000),
        );
        set_timeout(
            move || {
                set_toasts.update(|t| {
                    t.retain(|t| t.id != curr_id);
                });
            },
            std::time::Duration::from_millis(3300),
        );
        id.update_value(|id| *id += 1);
    });
    provide_context(ToastContext { push_toast });

    view! {
        {children()}
        <Portal mount=document().body().unwrap()>
            <div class="fixed bottom-8 right-4 flex flex-col-reverse gap-2">
                <For
                    each=toasts
                    key=|t| t.id
                    children=move |t| {
                        view! { <Toast toast=t /> }
                    }
                />
            </div>
        </Portal>
    }
}

pub fn use_toast() -> Callback<String> {
    use_context::<ToastContext>().unwrap().push_toast
}

#[component]
fn Toast(toast: Toast) -> impl IntoView {
    logging::log!("info: {}", toast.message);
    view! {
        <Surface class=move || {
            format!(
                "px-4 py-2 {}",
                if toast.is_closing.get() {
                    "animate-fade-out"
                } else {
                    "animate-slide-in-from-right"
                },
            )
        }>{toast.message}</Surface>
    }
}
