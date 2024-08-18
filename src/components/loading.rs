use leptos::*;
use leptos_use::use_raf_fn;
use web_sys::js_sys;

use crate::{draw::GolCanvas, parse::rle, universe::Universe};

use super::create_2d_context;

type LoadingContext = ReadSignal<Option<GolCanvas>>;

pub fn create_loading_canvas() -> ReadSignal<Option<GolCanvas>> {
    let canvas_ref = create_node_ref::<html::Canvas>();
    let universe = store_value({
        let u = Universe::with_size(5);

        let rle = include_str!("../../public/patterns/clock2.rle");
        let rect = rle::to_rect(rle).unwrap();
        u.root.borrow_mut().set_rect(-6, -6, &rect);
        u
    });
    let (canvas, set_canvas) = create_signal::<Option<GolCanvas>>(None);

    create_effect(move |_| {
        let canvas = canvas_ref().unwrap();

        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"alpha".into(), &false.into()).unwrap();
        set_canvas(Some(create_2d_context(canvas, options)));
    });

    let prev_tick = store_value(0.0);
    use_raf_fn(move |raf_args| {
        let now = raf_args.timestamp;
        if now - prev_tick() < 1000.0 / 4.0 {
            return;
        }
        universe.update_value(|u| u.step());
        prev_tick.set_value(now);
        set_canvas.update(|gc| {
            let gc = gc.as_mut().unwrap();
            gc.fit_rect(-6.0, -6.0, 12.0, 12.0);
            gc.clear();
            universe.with_value(|u| {
                let root = u.root.borrow();
                let half = (1 << (root.level - 1)) as f64;
                gc.draw_node(&root, -half - gc.origin.1, -half - gc.origin.0);
            });
        });
    });

    let _ = view! { <canvas _ref=canvas_ref width=64 height=64></canvas> };
    canvas
}

#[component]
pub fn Loading() -> impl IntoView {
    let canvas_ref = create_node_ref::<html::Canvas>();
    let global_canvas = use_context::<LoadingContext>().unwrap();
    let (canvas, set_canvas) = create_signal::<Option<GolCanvas>>(None);
    create_effect(move |_| {
        let canvas = canvas_ref().unwrap();

        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"alpha".into(), &false.into()).unwrap();
        set_canvas(Some(create_2d_context(canvas, options)));
    });

    create_effect(move |_| {
        canvas.with(|c| {
            if let Some(c) = c.as_ref() {
                let gc = global_canvas.with(|gc| gc.as_ref().unwrap().ctx.canvas().unwrap());
                c.ctx
                    .draw_image_with_html_canvas_element(&gc, 0.0, 0.0)
                    .unwrap();
            }
        });
    });

    view! { <canvas _ref=canvas_ref width=64 height=64></canvas> }
}
