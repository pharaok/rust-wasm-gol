use super::PatternCard;
use gloo_net::http::Request;
use leptos::*;
use leptos_use::{use_infinite_scroll_with_options, UseInfiniteScrollOptions};

#[component]
pub fn PatternLibrary() -> impl IntoView {
    let div_ref = create_node_ref::<html::Div>();
    let (name_index, set_name_index) = create_signal(10);
    let _ = use_infinite_scroll_with_options(
        div_ref,
        move |_| async move {
            set_name_index.update(|i| *i += 4);
        },
        UseInfiniteScrollOptions::default().distance(200.0),
    );

    let names = create_resource(
        || (),
        move |_| async {
            let resp = Request::get("/names.json").send().await.unwrap();
            resp.json::<Vec<String>>().await.unwrap()
        },
    );
    let shown = move || {
        names()
            .map(|names| {
                names
                    .into_iter()
                    .enumerate()
                    .take(name_index())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    view! {
        <Transition>

            {
                view! {
                    <div _ref=div_ref class="h-screen overflow-y-scroll p-1 flex flex-col gap-1">
                        <For
                            each=shown
                            key=|(_, name)| name.clone()
                            children=move |(_, name)| {
                                view! { <PatternCard name=name/> }
                            }
                        />

                    </div>
                }
            }

        </Transition>
    }
}
