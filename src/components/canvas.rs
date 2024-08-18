use leptos::*;
use leptos_use::{use_debounce_fn_with_arg, use_resize_observer};
use web_sys::{js_sys, wasm_bindgen::JsCast, CanvasRenderingContext2d};

use crate::draw::GolCanvas;

pub fn create_2d_context(canvas: HtmlElement<html::Canvas>, options: js_sys::Object) -> GolCanvas {
    let ctx = canvas
        .get_context_with_context_options("2d", &options)
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    GolCanvas::new(ctx)
}

#[component]
pub fn Canvas(
    canvas: ReadSignal<Option<GolCanvas>>,
    set_canvas: WriteSignal<Option<GolCanvas>>,
) -> impl IntoView {
    let div_ref = create_node_ref::<html::Div>();
    let canvas_ref = create_node_ref::<html::Canvas>();

    create_effect(move |_| {
        let canvas = canvas_ref().unwrap();
        canvas.set_width(div_ref().unwrap().client_width() as u32);
        canvas.set_height(div_ref().unwrap().client_height() as u32);
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"alpha".into(), &false.into()).unwrap();

        set_canvas(Some(create_2d_context(canvas, options)));
    });

    let debounced_resize = use_debounce_fn_with_arg(
        move |(width, height): (u32, u32)| {
            let canvas = canvas_ref().unwrap();
            canvas.set_width(width);
            canvas.set_height(height);
        },
        100.0,
    );
    use_resize_observer(div_ref, move |entries, _observer| {
        let rect = entries[0].content_rect();
        debounced_resize((rect.width() as u32, rect.height() as u32));
    });

    view! {
        <div _ref=div_ref class="absolute overflow-hidden w-full h-full bg-black">
            <canvas _ref=canvas_ref tabindex=0 class="absolute"></canvas>
        </div>
    }
}
