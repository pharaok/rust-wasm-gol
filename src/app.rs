use gloo_net::http::Request;
use leptos::*;
use leptos_router::{use_params, Params};

use crate::{
    components::{Canvas, Controls, Status},
    draw::GolCanvas,
    parse::rle,
    quadtree::Node,
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

pub async fn fetch_pattern(name: String) -> Result<String, ()> {
    if name.is_empty() {
        return Err(());
    }
    let url = format!("/patterns/{}.rle", name);
    let resp = Request::get(&url).send().await.map_err(|_| ())?;
    resp.text().await.map_err(|_| ())
}

#[component]
pub fn App() -> impl IntoView {
    let (universe, set_universe) = create_signal(Universe::default());
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
    let pattern_rle = create_resource(pattern_name, fetch_pattern);
    create_effect(move |_| {
        // pattern_rle will never actually be Some(Err) because
        // the server will always return 200 OK since this is a SPA
        if let Some(Ok(rle)) = pattern_rle() {
            if let Ok(rect) = rle::to_rect(&rle) {
                let (w, h) = (rect[0].len() as i32, rect.len() as i32);
                set_universe.update(|u| {
                    let mut root = u.root.borrow_mut();
                    *root = Node::new(root.level);
                    root.set_rect(-w / 2, -h / 2, &rect);
                });
                set_canvas.update(|gc| {
                    let gc = gc.as_mut().unwrap();
                    gc.fit_rect((-w / 2) as f64, (-h / 2) as f64, w as f64, h as f64);
                    gc.zoom_at_center(0.8);
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
