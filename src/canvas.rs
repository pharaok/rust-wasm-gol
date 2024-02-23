use std::collections::HashMap;

use leptos::{leptos_dom::logging::console_log, *};
use web_sys::{
    wasm_bindgen::{JsCast, JsValue},
    CanvasRenderingContext2d,
};

type Grid = HashMap<(i32, i32), u8>;

pub fn neighbor_count(grid: &Grid, x: i32, y: i32) -> i32 {
    let mut count = 0;
    for nx in (x - 1)..=(x + 1) {
        for ny in (y - 1)..=(y + 1) {
            if !(nx == x && ny == y) && grid.get(&(nx, ny)).is_some_and(|v| *v == 1) {
                count += 1;
            }
        }
    }
    count
}

pub fn tick(grid: &Grid) -> Grid {
    let mut new_grid = Grid::new();

    for ((cx, cy), _) in grid {
        for x in (cx - 1)..=(cx + 1) {
            for y in (cy - 1)..=(cy + 1) {
                if !new_grid.contains_key(&(x, y)) {
                    match neighbor_count(grid, x, y) {
                        2 => new_grid.insert((x, y), *grid.get(&(x, y)).unwrap_or(&0)),
                        3 => new_grid.insert((x, y), 1),
                        _ => None,
                    };
                }
            }
        }
    }

    new_grid
}

#[component]
pub fn Canvas() -> impl IntoView {
    let canvas_ref = create_node_ref::<html::Canvas>();
    let context = store_value::<Option<CanvasRenderingContext2d>>(None);

    let inner_width = window().inner_width().unwrap().as_f64().unwrap();
    let inner_height = window().inner_height().unwrap().as_f64().unwrap();

    let (grid, set_grid) = create_signal(Grid::new());
    let (origin, set_origin) = create_signal((1.0, 1.0));
    let (cell_size, set_cell_size) = create_signal(32.0);
    let pinned = store_value::<Option<(f64, f64)>>(None);

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

        context().unwrap()
    });

    create_effect(move |_| {
        let ctx = context().unwrap();
        let (o_x, o_y) = origin();

        ctx.set_fill_style(&JsValue::from_str("black"));
        ctx.fill_rect(0.0, 0.0, inner_width, inner_height);

        ctx.set_fill_style(&JsValue::from_str("white"));
        for ((x, y), v) in grid() {
            if v == 1 {
                ctx.fill_rect(
                    (x as f64 - o_x) * cell_size(),
                    (y as f64 - o_y) * cell_size(),
                    cell_size(),
                    cell_size(),
                );
            }
        }
    });

    view! {
        <canvas
            _ref=canvas_ref
            tabindex=0
            on:pointerdown=move |ev| {
                let (o_x, o_y) = origin();
                let grid_x = ev.offset_x() as f64 / cell_size() + o_x;
                let grid_y = ev.offset_y() as f64 / cell_size() + o_y;
                match ev.button() {
                    0 => {
                        let c = (grid_x.floor() as i32, grid_y.floor() as i32);
                        let cell = grid().get(&c).unwrap_or(&0).clone();
                        set_grid
                            .update(|grid| {
                                grid.insert(c, if cell == 0 { 1 } else { 0 });
                            });
                    }
                    1 => {
                        pinned.set_value(Some((grid_x, grid_y)));
                        ev.prevent_default();
                    }
                    _ => panic!(),
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
                set_grid
                    .update(|grid| {
                        *grid = tick(&grid);
                    });
            }
        >
        </canvas>
    }
}
