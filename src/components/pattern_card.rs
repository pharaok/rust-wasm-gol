use leptos::prelude::*;
use leptos_router::components::*;

use crate::{
    app::fetch_pattern,
    components::{Loading, Text},
    parse::rle::{self, PatternMetadata},
};

#[component]
pub fn PatternCard(#[prop(into)] name: String) -> impl IntoView {
    let value = name.clone(); // HACK: ?
    let pattern_rle = LocalResource::new(move || fetch_pattern(value.clone()));
    let pattern_metadata = move || {
        pattern_rle
            .get()
            .map(|resp| resp.and_then(|rle| rle::parse_metadata(rle.as_ref())))
    };

    view! {
        <div class="rounded-md bg-neutral-800 w-64 p-2">
            {move || {
                if let Some(
                    Ok((PatternMetadata { name: title, comment, owner, width, height, .. }, _)),
                ) = pattern_metadata() {
                    view! {
                        <A href=format!("/{}", name.clone())>
                            <h2 class="text-lg font-bold w-full text-center truncate">
                                {title.unwrap_or("No title".to_string())}
                            </h2>
                            <div class="w-full aspect-square flex justify-center items-center bg-black">
                                <Loading />
                            </div>
                        </A>
                        <div class="overflow-hidden">
                            <Text text=comment />
                        </div>
                        <div class="w-full">
                            {owner.map(|o| view! { <p>{format!("Author: {}", o)}</p> })}
                            <p>{format!("Size: {}x{}", width, height)}</p>
                        </div>
                    }
                        .into_view();
                } else {
                    view! {
                        <h2 class="text-lg font-bold w-full text-center truncate">
                            {name.clone()}
                        </h2>
                        <div class="w-full aspect-square flex justify-center items-center bg-black">
                            <Loading />
                        </div>
                    }
                        .into_view();
                }
            }}

        </div>
    }
}
