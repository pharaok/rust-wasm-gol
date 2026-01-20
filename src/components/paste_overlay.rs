use leptos::prelude::*;

use crate::{app::GolContext, components::Layer, draw, universe::Universe};

#[derive(Clone)]
pub struct ClipboardContext {
    pub paste_universe: RwSignal<Universe, LocalStorage>,
    pub paste_size: RwSignal<(i64, i64), LocalStorage>,
    pub is_pasting: RwSignal<bool, LocalStorage>,
}

#[component]
pub fn PasteLayer() -> impl IntoView {
    let GolContext {
        viewport,
        canvas_size,
        cursor,
        ..
    } = use_context::<GolContext>().unwrap();
    let ClipboardContext {
        paste_universe,
        paste_size,
        is_pasting,
    } = use_context::<ClipboardContext>().unwrap();

    let is_paste_canvas_dirty = StoredValue::new_local(false);
    Effect::new(move |_| {
        is_pasting.track();
        paste_size.track();
        paste_universe.track();
        cursor.track();
        canvas_size.track();
        viewport.track();
        is_paste_canvas_dirty.set_value(true);
    });

    view! {
        <Layer draw=move |c, _raf_args| {
            if !is_paste_canvas_dirty.get_value() {
                return;
            }
            if !is_pasting.get() {
                c.clear();
                c.draw();
                return;
            }
            let (width, height) = paste_size.get();
            let mut vp = viewport.get();
            vp.origin.0 -= cursor.get().0.floor();
            vp.origin.1 -= cursor.get().1.floor();
            c.clear();
            c.fill_rect_with_viewport(&vp, 0.0, 0.0, width as f64, height as f64, 0x00FFFF3F);
            paste_universe
                .with(|u| {
                    draw::draw_node(c, &vp, u, 0x00FFFFBF);
                });
            is_paste_canvas_dirty.set_value(false);
        } />
    }
}
