use crate::app::{GolContext, use_fit_universe};
use leptos::prelude::*;
use leptos_use::{UseTimeoutFnReturn, use_timeout_fn};

#[component]
fn Divider() -> impl IntoView {
    view! { <div class="border-l border-neutral-700"></div> }
}

#[component]
fn Item(
    children: Children,
    #[prop(into, optional)] on_press: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <span
            class=format!(
                "px-4 py-1 {}",
                if on_press.is_some() {
                    "cursor-pointer hover:bg-white/10 pointer-events-auto"
                } else {
                    ""
                },
            )

            on:click=move |_| {
                if let Some(on_press) = &on_press {
                    on_press.run(());
                }
            }
        >

            {children()}
        </span>
    }
}

fn metric_string(n: f64) -> String {
    if n >= 1e15 {
        format!("{:.1}P", n / 1e15)
    } else if n >= 1e12 {
        format!("{:.1}T", n / 1e12)
    } else if n >= 1e9 {
        format!("{:.1}G", n / 1e9)
    } else if n >= 1e6 {
        format!("{:.1}M", n / 1e6)
    } else if n >= 1e3 {
        format!("{:.1}k", n / 1e3)
    } else {
        format!("{:.1}", n)
    }
}

#[component]
pub fn Status() -> impl IntoView {
    let GolContext {
        universe,
        name,
        cursor,
        viewport,
        ..
    } = use_context::<GolContext>().unwrap();
    let ratio = move || {
        let cell_size = viewport.get().cell_size;
        if cell_size < 1.0 {
            format!("1:{}", metric_string(1.0 / cell_size))
        } else {
            format!("{}:1", metric_string(cell_size))
        }
    };

    let (is_renaming, set_is_renaming) = signal(false);
    let UseTimeoutFnReturn {
        start,
        stop,
        is_pending,
        ..
    } = use_timeout_fn(
        |_| {
            todo!("info");
        },
        300.0,
    );

    view! {
        <div class="flex flex-wrap justify-between items-center text-sm">
            <div
                class="cursor-pointer pointer-events-auto"
                on:click=move |_| {
                    if is_pending.get() {
                        stop();
                        set_is_renaming.set(true);
                    } else {
                        start(());
                    }
                }
            >
                {move || {
                    if is_renaming.get() {
                        view! {
                            <input
                                class="bg-transparent text-white px-4 py-1"
                                on:input=move |e| {
                                    name.set(event_target_value(&e));
                                }
                                on:blur=move |_| {
                                    set_is_renaming.set(false);
                                }
                                prop:value=name.get()
                            />
                        }
                            .into_any()
                    } else {

                        view! { <Item on_press=move || {}>{move || name.get()}</Item> }
                            .into_any()
                    }
                }}

            </div>
            <div class="ml-auto inline-flex flex-wrap">
                <Item>{move || format!("Step: {}", 1i64 << universe.with(|u| u.step))}</Item>
                <Divider />
                <Item>{move || format!("Gen: {}", universe.with(|u| u.generation))}</Item>
                <Divider />
                <Item>{move || format!("Pop: {}", universe.with(|u| u.population()))}</Item>
                <Divider />
                <Item on_press=move || {
                    use_fit_universe();
                }>{ratio}</Item>
                <Divider />
                <Item>
                    {move || {
                        format!(
                            "{}, {}",
                            cursor.get().0.floor() as i64,
                            cursor.get().1.floor() as i64,
                        )
                    }}

                </Item>
            </div>
        </div>
    }
}
