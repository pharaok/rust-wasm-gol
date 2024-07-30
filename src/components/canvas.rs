use leptos::*;
use leptos_use::use_raf_fn;
use web_sys::{
    wasm_bindgen::{JsCast, JsValue},
    CanvasRenderingContext2d,
};

use crate::{draw::GolCanvas, universe::Universe};

#[component]
pub fn Canvas() -> impl IntoView {
    let canvas_ref = create_node_ref::<html::Canvas>();
    let gol_canvas = store_value::<Option<GolCanvas>>(None);
    let offset_to_grid = move |x: i32, y: i32| {
        gol_canvas.with_value(|gc| gc.as_ref().unwrap().to_grid(x as f64, y as f64))
    };
    let pan = store_value::<Option<(f64, f64)>>(None);

    let (universe, set_universe) = create_signal(Universe::new()); // expensive to clone

    let inner_width = window().inner_width().unwrap().as_f64().unwrap();
    let inner_height = window().inner_height().unwrap().as_f64().unwrap();

    create_effect(move |_| {
        let canvas = canvas_ref().unwrap();
        canvas.set_width(inner_width as u32);
        canvas.set_height(inner_height as u32);

        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let mut gc = GolCanvas {
            ctx,
            ox: 0.0,
            oy: 0.0,
            cell_size: 20.0,
        };
        gc.ox = -gc.width() / 2.0;
        gc.oy = -gc.height() / 2.0;

        gol_canvas.set_value(Some(gc));
    });

    use_raf_fn(move |_raf_args| {
        gol_canvas.with_value(|gc| {
            let gc = gc.as_ref().unwrap();
            let ctx = &gc.ctx;
            ctx.set_fill_style(&JsValue::from_str("black"));
            ctx.fill_rect(0.0, 0.0, inner_width, inner_height);
            ctx.set_fill_style(&JsValue::from_str("white"));

            universe.with_untracked(|u| {
                let root = u.root.borrow();
                let half = (1 << (root.level - 1)) as f64;
                gc.draw_node(&root, -half - gc.oy, -half - gc.ox);
            });
        });
    });

    view! {
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
                    gol_canvas
                        .update_value(|gc| {
                            let gc = gc.as_mut().unwrap();
                            gc.ox += px - x;
                            gc.oy += py - y;
                        })
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
                gol_canvas
                    .update_value(|gc| {
                        gc.as_mut().unwrap().zoom_at(factor, x, y);
                    })
            }

            on:keydown=move |ev| {
                if ev.key().as_str() == " " {
                    set_universe
                        .update(|u| {
                            u.step(0);
                        });
                }
            }
        >
        </canvas>
    }
}
