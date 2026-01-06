use crate::{
    components::{Canvas, Controls, Menu, MenuTrigger, PatternLibrary, Status},
    draw::GolCanvas,
    parse::rle::{self, PatternMetadata},
    universe::Universe,
};
use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_router::hooks::*;
use leptos_router::params::Params;
use leptos_use::use_raf_fn;

#[derive(Params, PartialEq)]
pub struct GolParams {
    pub name: Option<String>,
}

#[derive(Clone)]
pub struct GolContext {
    pub universe: ReadSignal<Universe, LocalStorage>,
    pub set_universe: WriteSignal<Universe, LocalStorage>,
    pub cursor: ReadSignal<(f64, f64), LocalStorage>,
    pub set_cursor: WriteSignal<(f64, f64), LocalStorage>,
    pub canvas: ReadSignal<Option<GolCanvas>, LocalStorage>,
    pub set_canvas: WriteSignal<Option<GolCanvas>, LocalStorage>,
    pub is_ticking: ReadSignal<bool, LocalStorage>,
    pub set_is_ticking: WriteSignal<bool, LocalStorage>,
}

pub async fn fetch_pattern(name: String) -> Result<String, ()> {
    if name.is_empty() {
        return Err(());
    }
    let url = format!("/patterns/{}", name);
    let resp = Request::get(&url).send().await.map_err(|_| ())?;
    resp.text().await.map_err(|_| ())
}

#[component]
pub fn App(#[prop(optional, into)] meta: bool) -> impl IntoView {
    // WARN: a large Universe results in catastrophic cancellation in
    // draw_node, which causes issues with rendering and panning.
    let (universe, set_universe) =
        signal_local(Universe::with_size_and_arena_capacity(50, 1 << 24));
    let (canvas, set_canvas) = signal_local::<Option<GolCanvas>>(None);
    let (cursor, set_cursor) = signal_local((0.0, 0.0));
    let (is_ticking, set_is_ticking) = signal_local(false);
    let offset_to_grid =
        move |x: i32, y: i32| canvas.with(|gc| gc.as_ref().unwrap().to_grid(x as f64, y as f64));
    let pan = StoredValue::<Option<(f64, f64)>>::new(None);
    let tps = StoredValue::new(20.0);

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
    let pattern_name =
        move || params.with(|p| p.as_ref().unwrap().name.clone().unwrap_or_default());
    let pattern_rle = LocalResource::new(move || fetch_pattern(pattern_name()));

    let meta_on_rle = if meta {
        Some(LocalResource::new(move || {
            fetch_pattern("otcametapixelonb3s23.rle".to_owned())
        }))
    } else {
        None
    };
    let meta_off_rle = if meta {
        Some(LocalResource::new(move || {
            fetch_pattern("otcametapixeloffb3s23.rle".to_owned())
        }))
    } else {
        None
    };

    let is_dirty = StoredValue::new_local(true);
    Effect::new(move |_| {
        // pattern_rle will never actually be Some(Err) because
        // the server will always return 200 OK since this is a SPA
        if let Some(Ok(rle)) = pattern_rle.get()
            && let Ok((
                PatternMetadata {
                    width: w,
                    height: h,
                    ..
                },
                _,
            )) = rle::parse_metadata(&rle, "Unnamed Pattern", "")
        {
            set_universe.update(|u| {
                u.clear();
                if meta {
                    if let Some(Ok(on_rle)) = meta_on_rle.unwrap().get()
                        && let Some(Ok(off_rle)) = meta_off_rle.unwrap().get()
                    {
                        let rect = rle::to_rect(&rle).unwrap();
                        u.set_rect_meta(&rect, &on_rle, &off_rle);
                    }
                } else {
                    u.set_rle(-(w as i64) / 2, -(h as i64) / 2, &rle);
                }
            });
            set_canvas.update(|gc| {
                let gc = gc.as_mut().unwrap();
                if universe.with_untracked(|u| u.get_population()) != 0 {
                    let (t, l, b, r) = universe.with_untracked(|u| u.get_bounding_rect());
                    gc.fit_rect(l as f64, t as f64, (r - l + 1) as f64, (b - t + 1) as f64);
                }
                gc.zoom_at_center(0.6);
            });
        }
        is_dirty.set_value(true);
    });
    Effect::new(move |_| {
        universe.track();
        canvas.track();
        is_dirty.set_value(true);
    });

    let prev_tick = StoredValue::new_local(0.0);
    use_raf_fn(move |raf_args| {
        let now = raf_args.timestamp;
        if is_ticking.get() && now - prev_tick.get_value() > 1000.0 / tps.get_value() {
            set_universe.update(|u| {
                u.step();
            });
            prev_tick.set_value(now);
        }
        if !is_dirty.get_value() {
            return;
        }
        // NOTE: updates must be untracked, as otherwise the is_dirty flag gets
        // set back to true immediately.
        set_canvas.update_untracked(|gc| {
            let gc = gc.as_mut().unwrap();
            gc.clear();
            universe.with(|u| {
                let root = u.arena.get(u.root);
                let half = (1i64 << (root.level - 1)) as f64;
                gc.draw_node(u, -half - gc.origin.1, -half - gc.origin.0);
            });
        });
        is_dirty.set_value(false);
    });

    view! {
        <div class="absolute inset-0 w-screen h-screen">
            <div
                on:contextmenu=move |ev| ev.prevent_default()
                on:mousedown=move |ev| {
                    let (x, y) = offset_to_grid(ev.offset_x(), ev.offset_y());
                    match ev.button() {
                        0 => {
                            if canvas.with(|gc| gc.as_ref().unwrap().cell_size) < 5.0 {
                                return;
                            }
                            set_universe
                                .update(|u| {
                                    let (x, y) = (x.floor() as i64, y.floor() as i64);
                                    let v = u.get(x, y);
                                    u.set(x, y, v ^ 1);
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
                    if let Some((px, py)) = pan.get_value() {
                        set_canvas
                            .update(|gc| {
                                let gc = gc.as_mut().unwrap();
                                gc.origin.0 += px - x;
                                gc.origin.1 += py - y;
                            });
                    } else {
                        set_cursor.set((x, y));
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
                    set_canvas
                        .update(|gc| {
                            gc.as_mut().unwrap().zoom_at(factor, x, y);
                        });
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

                <Canvas canvas=canvas set_canvas=set_canvas />
            </div>
            <div on:click=|e| e.stop_propagation()>
                <div class="z-10 absolute bottom-4 left-[50%] -translate-x-[50%]">
                    <Controls />
                </div>
                <div class="absolute bottom-0 inset-x-0">
                    <Status />
                </div>
            </div>
            <Menu>
                <MenuTrigger>PATTERNS</MenuTrigger>
                <PatternLibrary />
            </Menu>
        </div>
    }
}
