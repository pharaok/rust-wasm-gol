use gloo_net::http::Request;
use leptos::{logging::log, *};
use leptos_router::{use_params, Params};
use leptos_use::use_raf_fn;

use crate::{
    components::{Canvas, Controls, Status},
    draw::GolCanvas,
    parse::rle,
    universe::Universe,
};

#[derive(Params, PartialEq)]
pub struct GolParams {
    pub name: Option<String>,
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
pub fn App() -> impl IntoView {
    let (universe, set_universe) = create_signal(Universe::new());
    let (canvas, set_canvas) = create_signal::<Option<GolCanvas>>(None);
    let (cursor, set_cursor) = create_signal((0.0, 0.0));
    let (is_ticking, set_is_ticking) = create_signal(false);

    provide_context(GolContext {
        universe,
        set_universe,
        cursor,
        set_cursor,
        canvas,
        set_canvas,
        is_ticking,
        set_is_ticking,
    });

    let params = use_params::<GolParams>();
    let pattern_name = move || {
        params.with(|p| {
            p.as_ref()
                .map(|p| p.name.clone().unwrap_or_default())
                .unwrap_or_default()
        })
    };
    let pattern_rle = create_resource(pattern_name, |name| async move {
        if name.is_empty() {
            return Err(());
        }
        let url = format!("/patterns/{}.rle", name);
        let resp = Request::get(&url).send().await.map_err(|_| ())?;
        resp.text().await.map_err(|_| ())
    });
    create_effect(move |_| {
        // pattern_rle will never actually be Some(Err) because
        // the server will always return 200 OK since this is a SPA
        if let Some(Ok(rle)) = pattern_rle() {
            if let Ok(grid) = rle::to_rect(&rle) {
                let (w, h) = (grid[0].len() as i32, grid.len() as i32);
                set_universe.update(|u| {
                    u.root.borrow_mut().set_rect(-w / 2, -h / 2, &grid);
                });
                set_canvas.update(|gc| {
                    let gc = gc.as_mut().unwrap();

                    let (inner_width, inner_height) = (
                        window().inner_width().unwrap().as_f64().unwrap(),
                        window().inner_height().unwrap().as_f64().unwrap(),
                    );
                    let new_cell_size = (inner_width / w as f64).min(inner_height / h as f64) * 0.8;
                    gc.zoom_at(new_cell_size / gc.cell_size, 0.0, 0.0);
                });
            }
        }
    });

    view! {
        <div class="relative w-screen h-screen">
            <Canvas/>
            <div class="z-10 absolute bottom-4 left-[50%] -translate-x-[50%]">
                <Controls/>
            </div>
            <div class="absolute bottom-0 inset-x-0">
                <Status/>
            </div>
        </div>
    }
}
