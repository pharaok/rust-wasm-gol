use crate::draw::Canvas;
use leptos::html;
use leptos::prelude::*;
use leptos_use::use_raf_fn;
use leptos_use::{UseRafFnCallbackArgs, use_debounce_fn_with_arg, use_resize_observer};
use web_sys::{CanvasRenderingContext2d, js_sys, wasm_bindgen::JsCast};

pub fn create_2d_context(canvas: web_sys::HtmlCanvasElement, options: js_sys::Object) -> Canvas {
    let ctx = canvas
        .get_context_with_context_options("2d", &options)
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    Canvas::new(ctx)
}

#[derive(Clone)]
pub struct StageContext {
    pub canvas_size: ReadSignal<(u32, u32), LocalStorage>,
}

#[component]
pub fn Stage(
    canvas_size: ReadSignal<(u32, u32), LocalStorage>,
    set_canvas_size: WriteSignal<(u32, u32), LocalStorage>,
    children: Children,
) -> impl IntoView {
    let div_ref = NodeRef::<html::Div>::new();

    let debounced_resize = use_debounce_fn_with_arg(
        move |size| {
            set_canvas_size(size);
        },
        100.0,
    );
    use_resize_observer(div_ref, move |entries, _observer| {
        let rect = entries[0].content_rect();
        debounced_resize((rect.width() as u32, rect.height() as u32));
    });
    provide_context(StageContext { canvas_size });

    view! {
        <div node_ref=div_ref class="absolute overflow-hidden inset-0 bg-black">
            {children()}
        </div>
    }
}

#[component]
pub fn Layer(draw: impl Fn(&mut Canvas, UseRafFnCallbackArgs) + 'static) -> impl IntoView {
    let StageContext { canvas_size } = use_context::<StageContext>().unwrap();
    let canvas_ref = NodeRef::<html::Canvas>::new();
    let (_canvas, set_canvas) = signal_local::<Option<Canvas>>(None);

    Effect::new(move |_| {
        let (width, height) = canvas_size.get();
        let canvas_el = canvas_ref.get().unwrap();
        canvas_el.set_width(width);
        canvas_el.set_height(height);
        let options = js_sys::Object::new();

        set_canvas.set(Some(create_2d_context(canvas_el, options)));
    });

    use_raf_fn(move |raf_args| {
        set_canvas.update(|c| {
            let c = c.as_mut().unwrap();
            draw(c, raf_args);
        });
    });

    view! { <canvas node_ref=canvas_ref tabindex=0 class="absolute"></canvas> }
}
