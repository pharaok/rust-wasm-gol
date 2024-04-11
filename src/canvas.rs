use leptos::{logging::log, *};
use leptos_use::{use_raf_fn, utils::Pausable};
use web_sys::{
    wasm_bindgen::{JsCast, JsValue},
    CanvasRenderingContext2d,
};

use crate::hashlife::{self, Node, NodeKind, Universe};

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

    let (grid, set_grid) = create_signal(Node::new(20));
    let universe = store_value(Universe::new());
    let (cell_size, set_cell_size) = create_signal::<f64>(32.0);
    let (origin, set_origin) = create_signal((0.0, 0.0));
    let pinned = store_value::<Option<(f64, f64)>>(None);

    let (is_ticking, set_is_ticking) = create_signal(false);
    let last_update = store_value(0.0);
    let tps = store_value(30.0);

    let Pausable {
        pause: _,
        resume: _,
        is_active: _,
    } = use_raf_fn(move |raf_args| {
        let ticks = if last_update() == 0.0 {
            1
        } else {
            ((raf_args.timestamp - last_update()) / (1000.0 / tps())) as i32
        };
        if is_ticking.get_untracked() && ticks > 0 {
            // set_grid.update(|grid| {
            //     // for _ in 0..ticks {
            //     //     *grid = tick(&grid);
            //     // }
            //     grid.grow();
            //     *grid = universe.get_value().step(grid);
            // });
            last_update.set_value(raf_args.timestamp);
        }

        let ctx = context().unwrap();
        let (o_x, o_y) = origin.get_untracked();
        let cell_size = cell_size.get_untracked();

        ctx.set_fill_style(&JsValue::from_str("#001133"));
        ctx.fill_rect(0.0, 0.0, inner_width, inner_height);

        ctx.set_fill_style(&JsValue::from_str("white"));

        let half = (1 << (grid.get_untracked().level - 1)) as f64;
        draw_node(
            &ctx,
            &grid.get_untracked(),
            -o_x - half,
            -o_y - half,
            cell_size,
        );

        ctx.set_font("30px sans-serif");
        ctx.fill_text(
            format!("generation: {}", grid.get_untracked().generation.get()).as_str(),
            30.0,
            50.0,
        )
        .unwrap();
        ctx.fill_text(
            format!("population: {}", grid.get_untracked().population.get()).as_str(),
            30.0,
            90.0,
        )
        .unwrap();
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
    });

    view! {
        <canvas
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
                        let cell = grid.get_untracked().get(cx, cy);
                        set_grid
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

            on:keydown=move |_| {
                set_is_ticking
                    .update(|s| {
                        *s = !*s;
                    });
                set_grid
                    .update(|grid| {
                        for _ in 0..1 {
                            grid.grow();
                            *grid = universe.get_value().step(grid, 0);
                        }
                    });
                last_update.set_value(0.0);
            }
        >
        </canvas>
    }
}
