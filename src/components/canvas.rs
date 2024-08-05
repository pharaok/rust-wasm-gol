use gloo_net::http::Request;
use leptos::{logging::log, *};
use leptos_router::{use_params, Params};
use leptos_use::{use_debounce_fn_with_arg, use_raf_fn, use_resize_observer};
use web_sys::{
    js_sys,
    wasm_bindgen::{JsCast, JsValue},
    CanvasRenderingContext2d
};

use crate::{
    components::{Controls, Status}, draw::GolCanvas, parse::rle, universe::Universe
};

#[derive(Params, PartialEq)]
struct GolParams {
    name: Option<String>,
}

#[derive(Clone)]
pub struct GolContext {
    pub universe: ReadSignal<Universe>,
    pub set_universe: WriteSignal<Universe>,
    pub cursor: ReadSignal<(f64, f64)>,
    pub set_cursor: WriteSignal<(f64, f64)>,
    pub canvas: ReadSignal<Option<GolCanvas>>,
    pub set_canvas: WriteSignal<Option<GolCanvas>>,
    pub is_ticking: ReadSignal<bool>,
    pub set_is_ticking: WriteSignal<bool>,
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

    let params = use_params::<GolParams>();
    let pattern_name = move || params.with(|p| {
        p.as_ref().map(|p| p.name.clone().unwrap_or_default()).unwrap_or_default()
    });
    let pattern_rle = create_resource(pattern_name, |name| async move {
        if name.is_empty() {
            return Err(())
        }
        let url = format!("/patterns/{}.rle",name);
        let resp = Request::get(&url).send().await.map_err(|_| ())?;
        resp.text().await.map_err(|_| ())
    });

    let (universe, set_universe) = create_signal(Universe::new());
    let (cursor, set_cursor) = create_signal((0.0, 0.0));
    let tps = store_value(20.0);
    let (is_ticking, set_is_ticking) = create_signal(false);
    provide_context(GolContext {
        universe,
        set_universe,
        cursor,
        set_cursor,
        canvas: gol_canvas,
        set_canvas: set_gol_canvas,
        is_ticking,
        set_is_ticking,
    });

    create_effect(move |_| {
        // pattern_rle will never actually be Some(Err) because
        // the server will always return 200 OK since this is a SPA
        if let Some(Ok(rle)) = pattern_rle() {
            if let Ok(grid) = rle::to_rect(&rle) {
                set_universe.update(|u| {
                    u.root.borrow_mut().set_rect(0, 0, &grid);
                })
            }
        }
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

    let prev_tick = store_value(0.0);
    use_raf_fn(move |raf_args| {
        let now = raf_args.timestamp;
        if is_ticking() && now - prev_tick() > 1000.0 / tps() {
            set_universe.update(|u| {
                u.step();
            });
            prev_tick.set_value(now);
        }

        gol_canvas.with(|gc| {
            let gc = gc.as_ref().unwrap();
            let ctx = &gc.ctx;
            gc.clear();

            ctx.set_fill_style(&JsValue::from_str("white"));
            universe.with(|u| {
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
                    let factor = std::f64::consts::E.powf(-ev.delta_y() / 1000.0);
                    set_gol_canvas
                        .update(|gc| {
                            gc.as_mut().unwrap().zoom_at(factor, x, y);
                        })
                }

                on:keydown=move |ev| {
                    match ev.key().as_str() {
                        " " => {
                            set_is_ticking.update(|b| *b = !*b);
                        }
                        _ => {}
                    }
                }
            >
            </canvas>
            <div class="z-10 absolute bottom-4 left-[50%] -translate-x-[50%]">
                <Controls/>
            </div>
            <div class="absolute bottom-0 inset-x-0">
                <Status/>
            </div>
        </div>
    }
}
