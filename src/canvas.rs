use std::collections::HashMap;

use leptos::{leptos_dom::logging::console_log, *};
use web_sys::{
    wasm_bindgen::{JsCast, JsValue},
    CanvasRenderingContext2d,
};

const CELL_SIZE: f64 = 32.0;

#[component]
pub fn Canvas() -> impl IntoView {
    let canvas_ref = create_node_ref::<html::Canvas>();
    let context = store_value::<Option<CanvasRenderingContext2d>>(None);

    let inner_width = window().inner_width().unwrap().as_f64().unwrap();
    let inner_height = window().inner_height().unwrap().as_f64().unwrap();

    let (grid, set_grid) = create_signal(HashMap::<(i32, i32), u8>::new());

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
        ctx.set_fill_style(&JsValue::from_str("black"));
        ctx.fill_rect(0.0, 0.0, inner_width, inner_height);

        ctx.set_fill_style(&JsValue::from_str("white"));
        for ((x, y), v) in grid() {
            if v == 1 {
                ctx.fill_rect(
                    x as f64 * CELL_SIZE,
                    y as f64 * CELL_SIZE,
                    CELL_SIZE,
                    CELL_SIZE,
                );
            }
        }
    });

    view! {
        <canvas
            _ref=canvas_ref
            on:pointerdown=move |ev| {
                let grid_x = ev.offset_x() as f64 / CELL_SIZE;
                let grid_y = ev.offset_y() as f64 / CELL_SIZE;
                let c = (grid_x as i32, grid_y as i32);
                let cell = grid().get(&c).unwrap_or(&0).clone();
                set_grid
                    .update(|grid| {
                        grid.insert(c, if cell == 0 { 1 } else { 0 });
                    });
            }
        >
        </canvas>
    }
}
