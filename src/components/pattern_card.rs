use crate::{
    app::fetch_pattern,
    components::{Loading, Text},
    draw::GolCanvas,
    parse::rle::{self, PatternMetadata},
};
use leptos::prelude::*;
use leptos_router::components::*;

#[component]
pub fn PatternCard(#[prop(into)] pattern: Signal<PatternMetadata, LocalStorage>) -> impl IntoView {
    let pattern_rle = LocalResource::new(move || fetch_pattern(pattern.get().path));

    let (canvas, set_canvas) = signal_local::<Option<GolCanvas>>(None);
    // Effect::new(move |_| {
    //     if let Some(Ok(rle)) = pattern_rle.get() {
    //         let rect = rle::to_rect(&rle).unwrap();
    //         logging::log!("{:?}", rect);
    //         if let Some(gc) = canvas.get() {
    //             gc.draw_rect(0.0, 0.0, 100.0, 100.0, rect);
    //             logging::log!("here");
    //         }
    //     }
    // });

    view! {
        <div class="rounded-md bg-neutral-800 w-64 p-2">
            <A href=format!("/{}", pattern.get().path)>
                <h2 class="text-lg font-bold w-full text-center truncate">{pattern.get().name}</h2>
                <div class="relative w-full aspect-square flex justify-center items-center bg-black">
                    <Show
                        when=move || true
                        fallback=move || {
                            view! { <Loading /> }
                        }
                    >
                        <Loading />
                    </Show>
                </div>
            </A>
            <div class="overflow-hidden">
                <Text text=pattern.get().comment />
            </div>
            <div class="w-full">
                {pattern.get().owner.map(|o| view! { <p>{format!("Author: {}", o)}</p> })}
                <p>{format!("Size: {}x{}", pattern.get().width, pattern.get().height)}</p>
            </div>
        </div>
    }
}
