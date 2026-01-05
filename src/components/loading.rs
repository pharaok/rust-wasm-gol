use super::create_2d_context;
use crate::{draw::GolCanvas, parse::rle, universe::Universe};
use leptos::html;
use leptos::prelude::*;
use leptos_use::use_raf_fn;
use web_sys::js_sys;

type LoadingContext = ReadSignal<Option<GolCanvas>, LocalStorage>;

#[component]
pub fn LoadingCanvasProvider(children: Children) -> impl IntoView {
    let canvas_ref = NodeRef::<html::Canvas>::new();
    let universe = StoredValue::new_local({
        let mut u = Universe::with_size_and_arena_capacity(5, 0);

        let rle = include_str!("../../public/patterns/clock2.rle");
        let rect = rle::to_rect(rle).unwrap();
        u.set_rect(-5, -5, &rect);
        u
    });
    let (canvas, set_canvas) = signal_local::<Option<GolCanvas>>(None);

    Effect::new(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            let options = js_sys::Object::new();
            js_sys::Reflect::set(&options, &"alpha".into(), &false.into()).unwrap();
            set_canvas.set(Some(create_2d_context(canvas, options)));
        }
    });

    let prev_tick = StoredValue::new_local(0.0);
    use_raf_fn(move |raf_args| {
        let now = raf_args.timestamp;
        if now - prev_tick.get_value() < 1000.0 / 4.0 {
            return;
        }
        universe.update_value(|u| u.step());
        prev_tick.set_value(now);
        if canvas.get().is_some() {
            set_canvas.update(|gc| {
                let gc = gc.as_mut().unwrap();
                gc.fit_rect(-6.0, -6.0, 12.0, 12.0);
                gc.clear();
                universe.with_value(|u| {
                    let half = (1i64 << (u.get_level() - 1)) as f64;
                    gc.draw_node(u, -half - gc.origin.1, -half - gc.origin.0);
                });
            });
        }
    });

    provide_context(canvas);

    view! {
        {children()}
        <canvas node_ref=canvas_ref width=64 height=64 style="display: none"></canvas>
    }
}

#[component]
pub fn Loading() -> impl IntoView {
    let canvas_ref = NodeRef::<html::Canvas>::new();
    let global_canvas = use_context::<LoadingContext>().unwrap();
    let (canvas, set_canvas) = signal_local::<Option<GolCanvas>>(None);

    Effect::new(move || {
        let canvas = canvas_ref.get().unwrap();

        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"alpha".into(), &false.into()).unwrap();
        set_canvas.set(Some(create_2d_context(canvas, options)));
    });

    Effect::new(move |_| {
        canvas.with(|c| {
            if let Some(c) = c.as_ref() {
                let gc = global_canvas.with(|gc| gc.as_ref().unwrap().ctx.canvas().unwrap());
                c.ctx
                    .draw_image_with_html_canvas_element(&gc, 0.0, 0.0)
                    .unwrap();
            }
        });
    });

    view! { <canvas node_ref=canvas_ref width=64 height=64></canvas> }
}
