use leptos::*;
use leptos_use::{use_debounce_fn_with_arg, use_raf_fn, use_resize_observer};
use web_sys::{
    js_sys,
    wasm_bindgen::{JsCast, JsValue},
    CanvasRenderingContext2d,
};

use crate::{components::Status, draw::GolCanvas, universe::Universe};

#[derive(Clone)]
pub struct GolContext {
    pub universe: ReadSignal<Universe>,
    pub set_universe: WriteSignal<Universe>,
    pub cursor: ReadSignal<(f64, f64)>,
    pub set_cursor: WriteSignal<(f64, f64)>,
    pub step: ReadSignal<i32>,
    pub set_step: WriteSignal<i32>,
    pub canvas: ReadSignal<Option<GolCanvas>>,
    pub set_canvas: WriteSignal<Option<GolCanvas>>,
}

#[component]
pub fn Canvas() -> impl IntoView {
    let div_ref = create_node_ref::<html::Div>();
    let canvas_ref = create_node_ref::<html::Canvas>();
    let (gol_canvas, set_gol_canvas) = create_signal::<Option<GolCanvas>>(None);
    let offset_to_grid = move |x: i32, y: i32| {
        gol_canvas.with(|gc| gc.as_ref().unwrap().to_grid(x as f64, y as f64))
    };
    let pan = store_value::<Option<(f64, f64)>>(None);

    let (universe, set_universe) = create_signal(Universe::new()); // WARN: expensive to clone
    let (step, set_step) = create_signal(0);
    let (cursor, set_cursor) = create_signal((0.0, 0.0));
    provide_context(GolContext {
        universe,
        set_universe,
        cursor,
        set_cursor,
        step,
        set_step,
        canvas: gol_canvas,
        set_canvas: set_gol_canvas,
    });

    create_effect(move |_| {
        let canvas = canvas_ref().unwrap();
        canvas.set_width(div_ref().unwrap().client_width() as u32);
        canvas.set_height(div_ref().unwrap().client_height() as u32);
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"alpha".into(), &false.into()).unwrap();

        let ctx = canvas
            .get_context_with_context_options("2d", &options)
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let gc = GolCanvas::new(ctx);
        set_gol_canvas(Some(gc));
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

    use_raf_fn(move |_raf_args| {
        gol_canvas.with_untracked(|gc| {
            let gc = gc.as_ref().unwrap();
            let ctx = &gc.ctx;
            gc.clear();

            ctx.set_fill_style(&JsValue::from_str("white"));
            universe.with_untracked(|u| {
                let root = u.root.borrow();
                let half = (1 << (root.level - 1)) as f64;
                gc.draw_node(&root, -half - gc.oy, -half - gc.ox);
            });
        });
    });

    view! {
        <div _ref=div_ref class="absolute overflow-hidden w-full h-full bg-black">
            <canvas
                _ref=canvas_ref
                tabindex=0
                class="absolute"
                on:contextmenu=move |ev| ev.prevent_default()
                on:mousedown=move |ev| {
                    let (x, y) = offset_to_grid(ev.offset_x(), ev.offset_y());
                    match ev.button() {
                        0 => {
                            set_universe
                                .update(|u| {
                                    let (x, y) = (x.floor() as i32, y.floor() as i32);
                                    let v = u.root.borrow().get(x, y);
                                    u.insert(x, y, v ^ 1);
                                });
                        }
                        1 => {
                            pan.set_value(Some((x, y)));
                        }
                        _ => {}
                    }
                }

                on:mousemove=move |ev| {
                    let (x, y) = offset_to_grid(ev.offset_x(), ev.offset_y());
                    if let Some((px, py)) = pan() {
                        set_gol_canvas
                            .update(|gc| {
                                let gc = gc.as_mut().unwrap();
                                gc.ox += px - x;
                                gc.oy += py - y;
                            })
                    } else {
                        set_cursor((x, y));
                    }
                }

                on:mouseup=move |ev| {
                    if ev.button() == 1 {
                        pan.set_value(None);
                    }
                }

                on:wheel=move |ev| {
                    let (x, y) = offset_to_grid(ev.offset_x(), ev.offset_y());
                    let factor = 1.0 - (ev.delta_y() / 1000.0);
                    set_gol_canvas
                        .update(|gc| {
                            gc.as_mut().unwrap().zoom_at(factor, x, y);
                        })
                }

                on:keydown=move |ev| {
                    if ev.key().as_str() == " " {
                        set_universe
                            .update(|u| {
                                u.step(step());
                            });
                    }
                }
            >
            </canvas>
            <div class="absolute bottom-0 right-0">
                <Status/>
            </div>
        </div>
    }
}
