use super::{Button, Icon, PatternCard};
use crate::components::ButtonVariant;
use crate::parse::rle::PatternMetadata;
use gloo_net::http::Request;
use leptos::html;
use leptos::prelude::*;
use leptos_use::{use_infinite_scroll_with_options, UseInfiniteScrollOptions};

#[derive(Clone)]
enum Sort {
    NameAsc,
    NameDesc,
    SizeAsc,
    SizeDesc,
}

#[component]
pub fn PatternLibrary() -> impl IntoView {
    let div_ref = NodeRef::<html::Div>::new();
    let (name_index, set_name_index) = signal(10);
    let _ = use_infinite_scroll_with_options(
        div_ref,
        move |_| async move {
            set_name_index.update(|i| *i += 5);
        },
        UseInfiniteScrollOptions::default().distance(200.0),
    );
    let (search, set_search) = signal(String::new());

    let patterns = LocalResource::new(move || async {
        let resp = Request::get("/patterns.json").send().await.unwrap();
        resp.json::<Vec<PatternMetadata>>().await.unwrap()
    });

    let (sort, set_sort) = signal_local(Sort::NameAsc);
    let by_name_asc = |a: &PatternMetadata, b: &PatternMetadata| a.name.cmp(&b.name);
    let by_name_desc = |a: &PatternMetadata, b: &PatternMetadata| b.name.cmp(&a.name);
    let by_size_asc = |a: &PatternMetadata, b: &PatternMetadata| {
        (a.width as u128 * a.height as u128).cmp(&(b.width as u128 * b.height as u128))
    };
    let by_size_desc = |a: &PatternMetadata, b: &PatternMetadata| {
        (b.width as u128 * b.height as u128).cmp(&(a.width as u128 * a.height as u128))
    };

    let shown = move || {
        patterns
            .get()
            .map(|ps| {
                let mut patterns = ps
                    .into_iter()
                    .filter(|p| p.name.to_lowercase().contains(&search()))
                    .collect::<Vec<_>>();
                patterns.sort_by(match sort.get() {
                    Sort::NameAsc => by_name_asc,
                    Sort::NameDesc => by_name_desc,
                    Sort::SizeAsc => by_size_asc,
                    Sort::SizeDesc => by_size_desc,
                });
                patterns.into_iter().take(name_index()).collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    Effect::new(move |_| {
        sort.get();
        set_name_index(10);
    });

    view! {
        <div node_ref=div_ref class="h-screen overflow-y-scroll p-1 flex flex-col gap-1">
            <div class="w-full flex gap-1">
                <input
                    class="w-0 flex-1 rounded-md bg-neutral-800"
                    placeholder="Search..."
                    type="text"
                    on:input=move |e| {
                        set_search(event_target_value(&e));
                        set_name_index(10);
                    }

                    prop:value=search
                />
                <Button
                    variant=ButtonVariant::Icon
                    class=move || {
                        format!(
                            "rounded-md {}",
                            match sort.get() {
                                Sort::SizeAsc => "bg-neutral-800",
                                Sort::SizeDesc => "bg-neutral-800",
                                _ => "",
                            },
                        )
                    }
                    on_press=move || {
                        set_sort
                            .update(|s| {
                                *s = match *s {
                                    Sort::SizeAsc => Sort::SizeDesc,
                                    _ => Sort::SizeAsc,
                                };
                            });
                    }
                >
                    {move || {
                        let icon = match sort.get() {
                            Sort::SizeDesc => icondata::LuArrowDownWideNarrow,
                            _ => icondata::LuArrowDownNarrowWide,
                        };
                        view! { <Icon icon=icon /> }
                    }}
                </Button>
                <Button
                    variant=ButtonVariant::Icon
                    class=move || {
                        format!(
                            "rounded-md {}",
                            match sort.get() {
                                Sort::NameAsc => "bg-neutral-800",
                                Sort::NameDesc => "bg-neutral-800",
                                _ => "",
                            },
                        )
                    }
                    on_press=move || {
                        set_sort
                            .update(|s| {
                                *s = match *s {
                                    Sort::NameAsc => Sort::NameDesc,
                                    _ => Sort::NameAsc,
                                };
                            });
                    }
                >
                    {move || {
                        let icon = match sort.get() {
                            Sort::NameDesc => icondata::LuArrowDownZA,
                            _ => icondata::LuArrowDownAZ,
                        };
                        view! { <Icon icon=icon /> }
                    }}
                </Button>
            </div>
            <Transition>
                <For
                    each=shown
                    key=|pattern| pattern.name.clone()
                    children=move |pattern| {
                        view! { <PatternCard pattern=pattern /> }
                    }
                />

            </Transition>
        </div>
    }
}
