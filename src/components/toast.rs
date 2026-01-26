use leptos::{logging, portal::Portal, prelude::*};

use crate::components::{Icon, Surface};

#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Info,
    Error,
}

#[derive(Clone, Debug)]
struct Toast {
    pub id: usize,
    pub message: String,
    pub level: LogLevel,
    is_closing: RwSignal<bool, LocalStorage>,
}

type PushToastType = Callback<(String, LogLevel)>;
#[derive(Clone, Copy)]
pub struct ToastContext {
    pub push_toast: PushToastType,
}
impl ToastContext {
    pub fn log(&self, msg: &str) {
        self.push_toast.run((msg.to_owned(), LogLevel::Info));
    }
    pub fn error(&self, msg: &str) {
        self.push_toast.run((msg.to_owned(), LogLevel::Error));
    }
}

#[component]
pub fn ToastRegion(children: Children) -> impl IntoView {
    let (toasts, set_toasts) = signal_local::<Vec<Toast>>(Vec::new());
    let id = StoredValue::new_local(0);

    let push_toast = Callback::new(move |(message, level): (String, LogLevel)| {
        let curr_id = id.get_value();
        set_toasts.update(|ts| {
            ts.push(Toast {
                id: curr_id,
                message,
                level,
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
            <div class="z-100 fixed bottom-8 right-4 flex flex-col-reverse gap-2">
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

pub fn use_toast() -> ToastContext {
    use_context::<ToastContext>().unwrap()
}

#[component]
fn Toast(toast: Toast) -> impl IntoView {
    match toast.level {
        LogLevel::Info => {
            logging::log!("info: {}", toast.message)
        }
        LogLevel::Error => {
            logging::error!("error: {}", toast.message)
        }
    };
    view! {
        <Surface class=move || {
            format!(
                "p-2 flex gap-2 items-center {} {}",
                if toast.is_closing.get() {
                    "animate-fade-out"
                } else {
                    "animate-slide-in-from-right"
                },
                match toast.level {
                    LogLevel::Info => "",
                    LogLevel::Error => "text-red-400",
                },
            )
        }>
            {match toast.level {
                LogLevel::Info => {
                    view! { <Icon icon=icondata::LuInfo /> }
                }
                LogLevel::Error => {
                    view! { <Icon icon=icondata::LuCircleAlert /> }
                }
            }} <span>{toast.message}</span>
        </Surface>
    }
}
