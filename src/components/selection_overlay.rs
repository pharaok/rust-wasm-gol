use leptos::prelude::*;

use crate::{app::GolContext, components::SelectionMenu};

#[component]
pub fn SelectionOverlay(#[prop(into)] is_open: Signal<bool, LocalStorage>) -> impl IntoView {
    let GolContext {
        selection_rect,
        viewport,
        canvas_size,
        ..
    } = use_context::<GolContext>().unwrap();

    view! {
        <div
            class="z-10 absolute flex justify-end items-start pointer-events-none"
            style:inset=move || {
                let vp = viewport.get();
                let (width, _) = canvas_size.get();
                if let Some((_, _, x2, y2)) = selection_rect.get() {
                    let (r, b) = vp.to_canvas_coords((x2 + 1) as f64, (y2 + 1) as f64);
                    format!("{}px {}px -9999px -9999px", b + 16, (width as i32) - r)
                } else {
                    "9999px".to_owned()
                }
            }
        >
            <Show when=is_open>
                <SelectionMenu />
            </Show>
        </div>
    }
}
