use leptos::*;

use crate::{
    app::fetch_pattern,
    components::Text,
    parse::rle::{self, PatternMetadata},
};

#[component]
pub fn PatternCard(#[prop(into)] name: String) -> impl IntoView {
    let pattern_rle = create_resource(move || name.clone(), fetch_pattern);
    let pattern_metadata = move || {
        pattern_rle
            .get()
            .map(|resp| resp.and_then(|rle| rle::parse_metadata(rle.as_ref())))
    };

    view! {
        {move || {
            if let Some(
                Ok((PatternMetadata { name: title, comment, owner, width, height, .. }, _)),
            ) = pattern_metadata() {
                view! {
                    <div class="rounded-md bg-neutral-800 w-64 p-2">
                        <h2 class="text-lg font-bold w-full text-center truncate">{title}</h2>
                        <div class="overflow-hidden">
                            <Text text=comment/>
                        </div>
                        <div class="w-full">
                            {owner
                                .map(|o| {
                                    view! { <p>{format!("Author: {}", o)}</p> }
                                })}
                            <p>{format!("Size: {}x{}", width, height)}</p>
                        </div>
                    </div>
                }
                    .into_view()
            } else {
                "Loading...".into_view()
            }
        }}
    }
}
