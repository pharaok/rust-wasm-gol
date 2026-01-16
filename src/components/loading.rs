use super::create_2d_context;
use crate::components::{Layer, Stage};
use crate::draw::{self, Viewport};
use crate::parse::rle;
use crate::universe::step_grid;
use leptos::html;
use leptos::prelude::*;
use web_sys::{CanvasRenderingContext2d, js_sys};

#[derive(Clone)]
pub struct LoadingContext {
    pub global_canvas_ref: NodeRef<html::Canvas>,
    pub frame: ReadSignal<u32, LocalStorage>,
}

#[component]
pub fn LoadingCanvasProvider(children: Children) -> impl IntoView {
    let canvas_ref = NodeRef::<html::Canvas>::new();
    let grid = StoredValue::new_local({
        let rle = include_str!("../../public/patterns/clock2.rle");
        let grid = rle::to_grid(rle).unwrap();
        let mut padded = vec![vec![0; grid[0].len() + 2]; grid.len() + 2];
        for (i, row) in grid.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                padded[1 + i][1 + j] = *cell;
            }
        }
        padded
    });

    let prev_tick = StoredValue::new_local(0.0);
    let (frame, set_frame) = signal_local(0);
    let (canvas_size, set_canvas_size) = signal_local((0, 0));

    provide_context(LoadingContext {
        global_canvas_ref: canvas_ref,
        frame,
    });

    view! {
        {children()}
        <div class="absolute opacity-0 -z-50 w-[72px] h-[72px]">
            <Stage canvas_size=canvas_size set_canvas_size=set_canvas_size>
                <Layer
                    node_ref=canvas_ref
                    draw=move |c, raf_args| {
                        let now = raf_args.timestamp;
                        if now - prev_tick.get_value() < 1000.0 / 4.0 {
                            return;
                        }
                        grid.update_value(|grid| {
                            let mut stepped = grid.clone();
                            step_grid(grid, &mut stepped);
                            std::mem::swap(grid, &mut stepped);
                            prev_tick.set_value(now);
                            let mut vp = Viewport::new();
                            vp.cell_size = 6.0;
                            vp.origin = (1.0, 1.0);
                            c.fill_rect(0, 0, c.width as i32, c.height as i32, 0x000000FF);
                            draw::draw_grid(c, &vp, grid);
                            set_frame.update(|f| *f += 1);
                        });
                    }
                />
            </Stage>
        </div>
    }
}

#[component]
pub fn Loading() -> impl IntoView {
    let canvas_ref = NodeRef::<html::Canvas>::new();
    let LoadingContext {
        global_canvas_ref,
        frame,
    } = use_context::<LoadingContext>().unwrap();
    let (ctx, set_ctx) = signal_local::<Option<CanvasRenderingContext2d>>(None);

    canvas_ref.on_load(move |canvas_el| {
        let options = js_sys::Object::new();
        set_ctx.set(Some(create_2d_context(canvas_el, options)));
    });

    Effect::new(move |_| {
        frame.track();
        ctx.with(|ctx| {
            if let Some(ctx) = ctx
                && let Some(gc) = global_canvas_ref.get()
            {
                let _ = ctx.draw_image_with_html_canvas_element(&gc, 0.0, 0.0);
            }
        })
    });

    view! { <canvas node_ref=canvas_ref width=72 height=72></canvas> }
}
