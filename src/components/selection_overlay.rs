use leptos::prelude::*;

use crate::{app::GolContext, components::Layer};

#[component]
pub fn SelectionOverlay() -> impl IntoView {
    let GolContext {
        selection_rect,
        viewport,
        canvas_size,
        ..
    } = use_context::<GolContext>().unwrap();
    let is_selection_dirty = StoredValue::new_local(true);
    Effect::new(move |_| {
        selection_rect.track();
        canvas_size.track();
        viewport.track();
        is_selection_dirty.set_value(true);
    });

    view! {
        <Layer draw=move |c, _raf_args| {
            let vp = viewport.get();
            if !is_selection_dirty.get_value() {
                return;
            }
            c.clear();
            if let Some((x1, y1, x2, y2)) = selection_rect.get() {
                c.fill_rect_with_viewport(
                    &vp,
                    x1 as f64,
                    y1 as f64,
                    (x2 - x1 + 1) as f64,
                    (y2 - y1 + 1) as f64,
                    0x0000FF7F,
                )
            }
            c.draw();
            is_selection_dirty.set_value(false);
        } />
    }
}
