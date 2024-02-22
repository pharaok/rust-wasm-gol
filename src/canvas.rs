use leptos::*;
use web_sys::{wasm_bindgen::JsCast, CanvasRenderingContext2d};

#[component]
pub fn Canvas() -> impl IntoView {
    let canvas_ref = create_node_ref::<html::Canvas>();
    let context = store_value::<Option<CanvasRenderingContext2d>>(None);

    let inner_width = window().inner_width().unwrap().as_f64().unwrap();
    let inner_height = window().inner_height().unwrap().as_f64().unwrap();

    create_effect(move |_| {
        canvas_ref().unwrap().set_width(inner_width as u32);
        canvas_ref().unwrap().set_height(inner_height as u32);
        context.set_value(Some(
            canvas_ref()
                .unwrap()
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap(),
        ));

        context().unwrap()
    });

    create_effect(move |_| {
        let ctx = context().unwrap();
        ctx.fill_rect(0.0, 0.0, inner_width, inner_height)
    });

    view! { <canvas _ref=canvas_ref></canvas> }
}
