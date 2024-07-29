use leptos::*;
use web_sys::{
    wasm_bindgen::{JsCast, JsValue},
    CanvasRenderingContext2d,
};

use crate::universe::Universe;

#[component]
pub fn Canvas() -> impl IntoView {
    let canvas_ref = create_node_ref::<html::Canvas>();
    let context = store_value::<Option<CanvasRenderingContext2d>>(None);

    let (universe, set_universe) = create_signal(Universe::new()); // expensive to clone

    let origin = store_value((0.0, 0.0));
    let cell_size = store_value(20.0);

    create_effect(move |_| {
        let inner_width = window().inner_width().unwrap().as_f64().unwrap();
        let inner_height = window().inner_height().unwrap().as_f64().unwrap();

        let canvas = canvas_ref().unwrap();
        canvas.set_width(inner_width as u32);
        canvas.set_height(inner_height as u32);

        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        ctx.set_fill_style(&JsValue::from_str("black"));
        ctx.fill_rect(0.0, 0.0, inner_width, inner_height);

        context.set_value(Some(ctx));
    });

    view! { <canvas _ref=canvas_ref on:contextmenu=move |ev| ev.prevent_default()></canvas> }
}
