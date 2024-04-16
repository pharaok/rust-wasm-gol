use std::time::Duration;

use leptos::{logging::log, *};
use leptos_use::{use_raf_fn, utils::Pausable};
use web_sys::{
    wasm_bindgen::{JsCast, JsValue},
    CanvasRenderingContext2d,
};

use crate::{
    button::Button,
    hashlife::{self, Node, NodeKind, Universe},
    icons::*,
};

fn draw_node(ctx: &CanvasRenderingContext2d, node: &Node, o_x: f64, o_y: f64, cell_size: f64) {
    if node.population.get() == 0 {
        return;
    }

    match &node.node {
        NodeKind::Leaf(_) => {
            if node.level != hashlife::LEAF_LEVEL {
                return;
            }
            for y in -1..1 {
                for x in -1..1 {
                    let cell = node.get(x, y);
                    if cell == 1 {
                        ctx.fill_rect(
                            ((o_x + x as f64 + 1.0) * cell_size).floor(),
                            ((o_y + y as f64 + 1.0) * cell_size).floor(),
                            (cell_size).ceil(),
                            (cell_size).ceil(),
                        );
                    }
                }
            }
        }
        NodeKind::Branch { nw, ne, sw, se } => {
            let half = (1 << (node.level - 1)) as f64;
            draw_node(ctx, &*nw.borrow(), o_x, o_y, cell_size);
            draw_node(ctx, &*ne.borrow(), o_x + half, o_y, cell_size);
            draw_node(ctx, &*sw.borrow(), o_x, o_y + half, cell_size);
            draw_node(ctx, &*se.borrow(), o_x + half, o_y + half, cell_size);
        }
    }
}

#[component]
pub fn Canvas() -> impl IntoView {
    let canvas_ref = create_node_ref::<html::Canvas>();
    let context = store_value::<Option<CanvasRenderingContext2d>>(None);

    let inner_width = window().inner_width().unwrap().as_f64().unwrap();
    let inner_height = window().inner_height().unwrap().as_f64().unwrap();

    let (root, set_root) = create_signal(Node::new(16));
    let universe = store_value(Universe::new());
    let (history, set_history) = create_signal(Vec::<Node>::new());

    let (cell_size, set_cell_size) = create_signal::<f64>(32.0);
    let (origin, set_origin) = create_signal((0.0, 0.0));
    let pinned = store_value::<Option<(f64, f64)>>(None);

    let (is_ticking, set_is_ticking) = create_signal(false);
    let last_update = store_value(0.0);
    let tps = store_value(10.0);
    let fps = store_value(0.0);
    let (shown_fps, set_shown_fps) = create_signal(0.0);
    let (step, set_step) = create_signal(0);

    let step_root = move || {
        if step.get_untracked() == -1 {
            if history.get_untracked().is_empty() {
                set_is_ticking(false);
                return;
            }
            set_history.update(|h| {
                set_root.update(|grid| {
                    *grid = h.pop().unwrap();
                });
            })
        } else {
            set_history.update(|h| {
                h.push(root.get_untracked().clone());
            });
            set_root.update(|grid| {
                grid.grow();
                *grid = universe.get_value().step(grid, step.get_untracked());
            });
        }
    };

    let Pausable {
        pause: _,
        resume: _,
        is_active: _,
    } = use_raf_fn(move |raf_args| {
        fps.set_value(1000.0 / raf_args.delta.round());

        let ticks = if last_update() == 0.0 {
            1
        } else {
            ((raf_args.timestamp - last_update()) * tps() / 1000.0) as i32
        };

        if is_ticking.get_untracked() && ticks > 0 {
            step_root();
            last_update.set_value(raf_args.timestamp);
        }

        let ctx = context().unwrap();
        let (o_x, o_y) = origin.get_untracked();
        let cell_size = cell_size.get_untracked();

        ctx.set_fill_style(&JsValue::from_str("black"));
        ctx.fill_rect(0.0, 0.0, inner_width, inner_height);

        let half = (1 << (root.get_untracked().level - 1)) as f64;
        let (w, h) = (inner_width / cell_size, inner_height / cell_size);

        // grid lines
        ctx.set_stroke_style(&JsValue::from_str("#101010"));
        ctx.begin_path();
        for x in 0..=w as i32 {
            ctx.move_to((x as f64 - o_x % 1.0) * cell_size, 0.0);
            ctx.line_to((x as f64 - o_x % 1.0) * cell_size, inner_height);
        }
        for y in 0..=h as i32 {
            ctx.move_to(0.0, (y as f64 - o_y % 1.0) * cell_size);
            ctx.line_to(inner_width, (y as f64 - o_y % 1.0) * cell_size);
        }
        ctx.close_path();
        ctx.stroke();

        ctx.set_fill_style(&JsValue::from_str("white"));
        draw_node(
            &ctx,
            &root.get_untracked(),
            -o_x - half,
            -o_y - half,
            cell_size,
        );
    });

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
        context()
            .unwrap()
            .set_fill_style(&JsValue::from_str("black"));
        context()
            .unwrap()
            .fill_rect(0.0, 0.0, inner_width, inner_height);

        set_interval(
            move || {
                set_shown_fps(fps());
            },
            Duration::new(1, 0),
        );
    });

    view! {
        <div class="relative w-screen h-screen">
            <canvas
                class="absolute inset-0"
                _ref=canvas_ref
                tabindex=0
                on:contextmenu=move |ev| {
                    ev.prevent_default();
                }

                on:pointerdown=move |ev| {
                    let (o_x, o_y) = origin();
                    let grid_x = ev.offset_x() as f64 / cell_size() + o_x;
                    let grid_y = ev.offset_y() as f64 / cell_size() + o_y;
                    match ev.button() {
                        0 => {
                            let (cx, cy) = (grid_x.floor() as i32, grid_y.floor() as i32);
                            let cell = root.get_untracked().get(cx, cy);
                            set_root
                                .update(|grid| {
                                    grid.insert(cx, cy, if cell == 0 { 1 } else { 0 });
                                });
                        }
                        2 => {
                            pinned.set_value(Some((grid_x, grid_y)));
                            ev.prevent_default();
                        }
                        _ => {}
                    }
                }

                on:pointermove=move |ev| {
                    if let Some((p_x, p_y)) = pinned() {
                        let (o_x, o_y) = origin();
                        let grid_x = ev.offset_x() as f64 / cell_size() + o_x;
                        let grid_y = ev.offset_y() as f64 / cell_size() + o_y;
                        let delta_x = grid_x - p_x;
                        let delta_y = grid_y - p_y;
                        set_origin((o_x - delta_x, o_y - delta_y));
                    }
                }

                on:pointerup=move |_| {
                    pinned.set_value(None);
                }

                on:wheel=move |ev| {
                    let factor = 1.0 + -(ev.delta_y() / 2000.0);
                    set_origin
                        .update(|origin| {
                            let (o_x, o_y) = origin.clone();
                            let grid_x = ev.offset_x() as f64 / cell_size();
                            let grid_y = ev.offset_y() as f64 / cell_size();
                            let delta_x = grid_x / factor - grid_x;
                            let delta_y = grid_y / factor - grid_y;
                            *origin = (o_x - delta_x, o_y - delta_y);
                        });
                    set_cell_size.update(|cs| *cs *= factor);
                }

                on:keydown=move |ev| {
                    log!("key: {}", ev.key());
                    match ev.key().as_str() {
                        " " => {
                            set_is_ticking.update(|s| *s = !*s);
                        }
                        _ => {}
                    }
                    last_update.set_value(0.0);
                }
            >
            </canvas>
            <div class="absolute inset-0 w-full h-full pointer-events-none [&>*]:pointer-events-auto">

                <div class="text-white absolute top-2 left-2 bg-white/10 rounded-lg p-2">
                    <div>{move || format!("gen: {}", root().generation.get())}</div>
                    <div>{move || format!("pop: {}", root().population.get())}</div>
                    <div>
                        {move || format!("step: {}", if step() == -1 { -1 } else { 1 << step() })}
                    </div>
                    <div>{move || format!("tps: {}", tps())}</div>
                    <div>{move || format!("fps: {}", shown_fps() as i32)}</div>

                </div>
                <div class="absolute bottom-2 left-[50%] -translate-x-[50%] bg-white/10 rounded-lg p-2">
                    <div class="flex gap-2 items-center">

                        <Button on_press=move || {
                            set_step.update(|s| *s = (*s - 1).max(-1));
                        }>
                            <Rewind/>
                        </Button>
                        <Button on_press=move || {
                            set_step.update(|s| *s = (*s + 1).min(root().level as i32));
                        }>
                            <FastForward/>
                        </Button>

                        <div class="h-8 bg-gray-300 w-px"></div>

                        <Button
                            disabled=Signal::derive(move || history().is_empty())
                            on_press=move || {
                                if history().is_empty() {
                                    return;
                                }
                                set_history
                                    .update(|h| {
                                        set_root(h.pop().unwrap());
                                    });
                            }
                        >

                            <StepBack/>
                        </Button>
                        <Button on_press=move || {
                            set_is_ticking.update(|s| *s = !*s)
                        }>
                            {move || {
                                if is_ticking() {
                                    view! { <Pause/> }
                                } else {
                                    view! { <Play/> }
                                }
                            }}

                        </Button>
                        <Button on_press=move || step_root()>
                            <StepForward/>
                        </Button>
                    </div>

                </div>
            </div>
        </div>
    }
}
