use crate::{
    app::fetch_pattern,
    components::{Layer, Loading, Stage, Text},
    draw,
    parse::rle::PatternMetadata,
};
use leptos::prelude::*;
use leptos_router::components::*;

#[component]
pub fn PatternCard(#[prop(into)] pattern: Signal<PatternMetadata, LocalStorage>) -> impl IntoView {
    let pattern_rle = LocalResource::new(move || fetch_pattern(pattern.get().path));

    let (canvas_size, set_canvas_size) = signal_local((0, 0));
    let (is_ready, set_is_ready) = signal_local(false);
    let is_dirty = StoredValue::new_local(false);
    Effect::new(move |_| {
        canvas_size.track();
        is_dirty.set_value(true);
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
                    <Stage canvas_size=canvas_size set_canvas_size=set_canvas_size>
                        <Layer draw=move |c, raf_args| {
                            if !is_dirty.get_value() {
                                return;
                            }
                            if let Some(Ok(rle)) = pattern_rle.get() {
                                draw::draw_rle(c, rle);
                                set_is_ready.set(true);
                                c.draw();
                                is_dirty.set_value(false);
                            }
                        } />
                    </Stage>
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
