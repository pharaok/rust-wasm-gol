use leptos::{logging::log, *};
use leptos_use::use_raf_fn;
use web_sys::{
    wasm_bindgen::{JsCast, JsValue},
    CanvasRenderingContext2d,
};

use crate::{
    quadtree::{Node, NodeKind},
    universe::Universe,
};

fn draw_node(gol_canvas: &GolCanvas, node: &Node, top: f64, left: f64) {
    if node.population.get() == 0 {
        return;
    }

    let half = (1 << (node.level - 1)) as f64;
    let (bottom, right) = (top + 2.0 * half, left + 2.0 * half);
    if bottom < 0.0 || right < 0.0 || top > gol_canvas.height() || left > gol_canvas.width() {
        return;
    }

    let cell_size = gol_canvas.cell_size;

    match &node.node {
        NodeKind::Leaf(leaf) => {
            // guaranteed to be at leaf level
            for (i, row) in leaf.iter().enumerate() {
                for (j, cell) in row.iter().enumerate() {
                    if *cell != 0 {
                        gol_canvas.ctx.fill_rect(
                            (left + j as f64) * cell_size,
                            (top + i as f64) * cell_size,
                            cell_size,
                            cell_size,
                        );
                    }
                }
            }
        }
        NodeKind::Branch([nw, ne, sw, se]) => {
            draw_node(gol_canvas, &nw.borrow(), top, left);
            draw_node(gol_canvas, &ne.borrow(), top, left + half);
            draw_node(gol_canvas, &sw.borrow(), top + half, left);
            draw_node(gol_canvas, &se.borrow(), top + half, left + half);
        }
    };
}

pub struct GolCanvas {
    pub ctx: CanvasRenderingContext2d,
    pub ox: f64,
    pub oy: f64,
    pub cell_size: f64,
}
impl GolCanvas {
    pub fn width(&self) -> f64 {
        self.ctx.canvas().unwrap().width() as f64 / self.cell_size
    }
    pub fn height(&self) -> f64 {
        self.ctx.canvas().unwrap().height() as f64 / self.cell_size
    }
    pub fn to_grid(&self, offset_x: f64, offset_y: f64) -> (f64, f64) {
        (
            (offset_x / self.cell_size) - self.ox,
            (offset_y / self.cell_size) - self.oy,
        )
    }
}

#[component]
pub fn Canvas() -> impl IntoView {
    let canvas_ref = create_node_ref::<html::Canvas>();
    let gol_canvas = store_value::<Option<GolCanvas>>(None);

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
            cell_size: 40.0,
        };
        gc.ox = -gc.width() / 2.0;
        gc.oy = -gc.height() / 2.0;

        gol_canvas.set_value(Some(gc));

        set_universe.update(|u| {
            u.insert(0, 0, 1);

            u.insert(0, 2, 1);
            u.insert(-1, 2, 1);

            u.insert(2, 1, 1);
            u.insert(3, 2, 1);
            u.insert(4, 2, 1);
            u.insert(5, 2, 1);

            for _ in 0..4 {
                u.step(1000);
            }
        })
    });

    use_raf_fn(move |_raf_args| {
        gol_canvas.with_value(|gc| {
            let ctx = &gc.as_ref().unwrap().ctx;
            ctx.set_fill_style(&JsValue::from_str("black"));
            ctx.fill_rect(0.0, 0.0, inner_width, inner_height);
            ctx.set_fill_style(&JsValue::from_str("white"));

            universe.with_untracked(|u| {
                let root = u.root.borrow();
                let half = (1 << (root.level - 1)) as f64;
                draw_node(
                    gc.as_ref().unwrap(),
                    &root,
                    -half - gc.as_ref().unwrap().oy,
                    -half - gc.as_ref().unwrap().ox,
                );
            });
        });
    });

    view! { <canvas _ref=canvas_ref on:contextmenu=move |ev| ev.prevent_default()></canvas> }
}
