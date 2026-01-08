use crate::{
    app::fetch_pattern,
    components::{Canvas, Loading, Text},
    draw::GolCanvas,
    parse::rle::PatternMetadata,
};
use leptos::prelude::*;
use leptos_router::components::*;

#[component]
pub fn PatternCard(#[prop(into)] pattern: Signal<PatternMetadata, LocalStorage>) -> impl IntoView {
    let pattern_rle = LocalResource::new(move || fetch_pattern(pattern.get().path));

    let (canvas, set_canvas) = signal_local::<Option<GolCanvas>>(None);
    let (is_ready, set_is_ready) = signal_local(false);
    Effect::new(move |_| {
        // track canvas to wait for initialization and resize.
        canvas.track();
        if let Some(Ok(rle)) = pattern_rle.get() {
            set_canvas.update_untracked(|gc| {
                if let Some(gc) = gc {
                    gc.draw_rle(rle);
                    set_is_ready.set(true);
                }
            });
        }
    });

    view! {
        <div class="rounded-md bg-neutral-800 w-64 p-2">
            <A href=format!("/{}", pattern.get().path)>
                <h2 class="text-lg font-bold w-full text-center truncate">{pattern.get().name}</h2>
                <div class="relative w-full aspect-square bg-black">
                    <Show when=move || !is_ready.get()>
                        <div class="absolute inset-0 z-10 flex justify-center items-center">
                            <Loading />
                        </div>
                    </Show>
                    <Canvas canvas=canvas set_canvas=set_canvas />
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
