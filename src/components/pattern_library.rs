use super::PatternCard;
use gloo_net::http::Request;
use leptos::html;
use leptos::prelude::*;
use leptos_use::{use_infinite_scroll_with_options, UseInfiniteScrollOptions};

#[component]
pub fn PatternLibrary() -> impl IntoView {
    let div_ref = NodeRef::<html::Div>::new();
    let (name_index, set_name_index) = signal(10);
    let _ = use_infinite_scroll_with_options(
        div_ref,
        move |_| async move {
            set_name_index.update(|i| *i += 4);
        },
        UseInfiniteScrollOptions::default().distance(200.0),
    );
    let (search, set_search) = signal(String::new());

    let names = LocalResource::new(move || async {
        let resp = Request::get("/names.json").send().await.unwrap();
        resp.json::<Vec<String>>().await.unwrap()
    });
    let shown = move || {
        names
            .get()
            .map(|names| {
                names
                    .into_iter()
                    .filter(|n| n.contains(&search()))
                    .take(name_index())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    view! {
        <div node_ref=div_ref class="h-screen overflow-y-scroll p-1 flex flex-col gap-1">
            <div>
                <input
                    class="w-full p-1 rounded-md bg-neutral-800"
                    placeholder="Search..."
                    type="text"
                    on:input=move |e| {
                        set_search(event_target_value(&e));
                        set_name_index(10);
                    }

                    prop:value=search
                />
            </div>
            <Transition>
                <For
                    each=shown
                    key=|name| name.clone()
                    children=move |name| {
                        view! { <PatternCard name=name /> }
                    }
                />

            </Transition>
        </div>
    }
}
