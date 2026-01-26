use crate::{
    components::{
        AppMenu, ClipboardContext, Controls, Layer, PasteLayer, SelectionLayer, SelectionOverlay,
        Stage, Status, use_toast,
    },
    draw::{self, Viewport},
    meta::use_metapixels,
    parse::rle::{self, PatternMetadata},
    universe::{InsertMode, Universe},
};
use gloo_net::http::Request;
use leptos::{ev::mousedown, html, logging, prelude::*};
use leptos_router::hooks::*;
use leptos_router::params::Params;
use leptos_use::{UseClipboardReturn, use_clipboard, use_document, use_event_listener};

#[derive(Params, PartialEq, Clone)]
pub struct GolParams {
    pub name: Option<String>,
}
#[derive(Params, PartialEq, Clone)]
pub struct GolQuery {
    pub rle: Option<String>,
}

#[derive(Clone, Copy)]
pub struct GolContext {
    pub universe: RwSignal<Universe, LocalStorage>,
    pub name: RwSignal<String, LocalStorage>,
    pub canvas_size: ReadSignal<(u32, u32), LocalStorage>,
    pub viewport: RwSignal<Viewport, LocalStorage>,
    pub cursor: RwSignal<(f64, f64), LocalStorage>,
    pub selection_rect: Signal<Option<(i64, i64, i64, i64)>, LocalStorage>,
    pub is_ticking: RwSignal<bool, LocalStorage>,
    pub tps: RwSignal<f64, LocalStorage>,
}

// TODO: impl GolContext
pub fn use_fit_universe() {
    let GolContext {
        universe,
        viewport,
        canvas_size,
        ..
    } = use_context::<GolContext>().unwrap();
    let (canvas_width, canvas_height) = canvas_size.get();
    if universe.with(|u| u.population()) != 0 {
        let (x1, y1, x2, y2) = universe.with(|u| u.get_bounding_rect());
        viewport.update(|vp| {
            vp.fit_rect(
                x1 as f64,
                y1 as f64,
                (x2 - x1 + 1) as f64,
                (y2 - y1 + 1) as f64,
                canvas_width as f64,
                canvas_height as f64,
            );
            vp.zoom_at_center(0.8, canvas_width as f64, canvas_height as f64);
        });
    }
}

pub type PatternResult = Result<String, ()>;
pub async fn fetch_pattern(name: String) -> PatternResult {
    if name.is_empty() {
        return Err(());
    }
    let url = format!("/patterns/{}", name);
    let resp = Request::get(&url).send().await.map_err(|_| ())?;
    resp.text().await.map_err(|_| ())
}

#[component]
pub fn App(#[prop(optional, into)] meta: bool) -> impl IntoView {
    let params = use_params::<GolParams>();
    let query = use_query::<GolQuery>();

    let universe = RwSignal::new_local(Universe::with_size_and_arena_capacity(60, 1 << 16));
    let (canvas_size, set_canvas_size) = signal_local((0, 0));
    let viewport = RwSignal::new_local(Viewport::new());
    let cursor = RwSignal::new_local((0.0, 0.0));
    let is_ticking = RwSignal::new_local(false);
    let tps = RwSignal::new_local(16.0);
    let offset_to_world = move |x: i32, y: i32| viewport.with(|vp| vp.to_world_coords(x, y));
    let pan = StoredValue::<Option<(f64, f64)>>::new(None);

    let (selection_start, set_selection_start) = signal_local::<Option<(i64, i64)>>(None);
    let (selection_end, set_selection_end) = signal_local::<Option<(i64, i64)>>(None);
    let (is_selection_menu_shown, set_is_selection_menu_shown) = signal_local(false);
    let selection_rect = Signal::derive_local(move || {
        let ((sx, sy), (ex, ey)) = (selection_start.get()?, selection_end.get()?);
        Some((sx.min(ex), sy.min(ey), sx.max(ex), sy.max(ey)))
    });
    let name = RwSignal::new_local({
        params.with(|p| {
            p.as_ref()
                .map(|p| p.name.clone().unwrap_or_default())
                .unwrap_or_default()
        })
    });

    let ctx = GolContext {
        universe,
        name,
        canvas_size,
        viewport,
        cursor,
        selection_rect,
        is_ticking,
        tps,
    };
    provide_context(ctx);

    let logging = use_toast();

    let pattern_name =
        move || params.with(|p| p.as_ref().unwrap().name.clone().unwrap_or_default());
    let pattern_rle = LocalResource::new(move || fetch_pattern(pattern_name()));

    let metapixels = use_metapixels();

    let did_fit = StoredValue::new_local(false);
    Effect::new(move |_| {
        pattern_rle.track();
        did_fit.set_value(false);
    });
    Effect::new(move |_| {
        logging::log!("here");
        let (canvas_width, canvas_height) = canvas_size.get();
        let param_rle = pattern_rle.get().and_then(Result::ok);
        let query_rle = query.with(|q| q.as_ref().ok().and_then(|q| q.rle.to_owned()));

        if did_fit.get_value() || canvas_width == 0 || canvas_height == 0 {
            return;
        }
        if let Some(rle) = param_rle.or(query_rle)
            && rle::parse_metadata(&rle, "", "").is_ok()
        {
            universe.update(|u| {
                u.clear();
                if meta {
                    if let Some((Ok(on_rle), Ok(off_rle))) = metapixels.get() {
                        let rect = rle::to_grid(&rle).unwrap();
                        u.set_grid_meta(&rect, &on_rle, &off_rle);
                    }
                } else {
                    let points = rle::iter_alive(&rle).unwrap().collect::<Vec<_>>();
                    let half = 1i64 << (u.level() - 1);
                    u.set_points(&points, -half, -half, half - 1, half - 1, &InsertMode::Copy);
                }
            });

            use_fit_universe();
            did_fit.set_value(true);
        }
    });

    let is_canvas_dirty = StoredValue::new_local(true);
    Effect::new(move |_| {
        universe.track();
        canvas_size.track();
        viewport.track();
        is_canvas_dirty.set_value(true);
    });

    let prev_tick = StoredValue::new_local(0.0);
    Effect::new(move |_| {
        is_ticking.track();
        if is_ticking.get() {
            universe.update(|u| {
                u.push_snapshot();
            });
        }
        prev_tick.set_value(0.0);
    });

    let keys = StoredValue::<Vec<String>, LocalStorage>::new_local(Vec::new());
    let did_pan = StoredValue::new_local(false);

    let UseClipboardReturn { copy, text, .. } = use_clipboard();

    let paste_universe = RwSignal::new_local(Universe::with_size_and_arena_capacity(30, 0));
    let paste_size = RwSignal::new_local((0, 0));
    let is_pasting = RwSignal::new_local(false);
    provide_context(ClipboardContext {
        paste_universe,
        paste_size,
        is_pasting,
    });

    let paste_rle = StoredValue::new_local(String::new());
    let is_paste_canvas_dirty = StoredValue::new_local(false);
    Effect::new(move |_| {
        is_pasting.track();
        paste_size.track();
        paste_universe.track();
        cursor.track();
        canvas_size.track();
        viewport.track();
        is_paste_canvas_dirty.set_value(true);
    });
    let copy_selection_rle = move || {
        if let Some((x1, y1, x2, y2)) = selection_rect.get() {
            universe.with(|u| {
                let rle = rle::from_iter(u.iter_alive_in_rect(x1, y1, x2, y2), x1, y1, x2, y2);
                copy(&rle);
                logging.log("Copied RLE to clipboard!");
            });
        }
    };
    let paste = move || {
        if let Ok(points) = rle::iter_alive(&paste_rle.get_value()) {
            let (cx, cy) = cursor.with(|(x, y)| (x.floor() as i64, y.floor() as i64));
            let (width, height) = paste_size.get();
            universe.update(|u| {
                u.push_snapshot();
                u.set_points(
                    &points.map(|(x, y)| (x + cx, y + cy)).collect::<Vec<_>>(),
                    cx,
                    cy,
                    cx + width - 1,
                    cy + height - 1,
                    &InsertMode::Or,
                );
            });
        }
        is_pasting.set(false);
    };
    let start_pasting = move |rle: &str| {
        is_pasting.set(true);
        if rle == paste_rle.get_value() {
            return;
        }
        paste_rle.set_value(rle.to_owned());
        if let Ok(points) = rle::iter_alive(rle) {
            paste_universe.update(|u| {
                let half = 1i64 << (u.level() - 1);
                u.set_points(
                    &points.collect::<Vec<_>>(),
                    -half,
                    -half,
                    half - 1,
                    half - 1,
                    &InsertMode::Copy,
                );
            });
        }
        if let Ok((PatternMetadata { width, height, .. }, _)) = rle::parse_metadata(rle, "", "") {
            paste_size.set((width as i64, height as i64));
        }
    };
    let toggle_cell = move |x: i64, y: i64| {
        universe.update(|u| {
            let v = u.get(x, y);
            u.set(x, y, v ^ 1);
        });
    };

    let div_ref = NodeRef::<html::Div>::new();
    div_ref.on_load(|div_el| {
        let _ = div_el.focus();
    });
    let _ = use_event_listener(use_document(), mousedown, move |ev| {
        let el = event_target::<web_sys::HtmlDivElement>(&ev);
        let tag = el.tag_name();

        if tag != "INPUT" && tag != "TEXTAREA" {
            let _ = div_ref.get().unwrap().focus();
            ev.prevent_default();
        }
    });

    view! {
        <div class="absolute top-0 left-0 w-full h-dvh overflow-hidden">
            <div
                tabindex="0"
                node_ref=div_ref
                on:contextmenu=move |ev| ev.prevent_default()
                on:mousedown=move |ev| {
                    if pan.get_value().is_some() {
                        return;
                    }
                    let (x, y) = offset_to_world(ev.offset_x(), ev.offset_y());
                    let is_space_held = keys.get_value().contains(&" ".to_owned());
                    match (ev.button(), is_space_held) {
                        (0, false) => {
                            set_selection_start.set(None);
                            set_selection_end.set(None);
                            set_is_selection_menu_shown.set(false);
                            if is_pasting.get() {
                                paste();
                            } else if viewport.get().cell_size >= 5.0 {
                                universe
                                    .update(|u| {
                                        u.push_snapshot();
                                    });
                                toggle_cell(x.floor() as i64, y.floor() as i64);
                            }
                        }
                        (1, _) | (0, true) => {
                            pan.set_value(Some((x, y)));
                            if is_space_held {
                                did_pan.set_value(true);
                            }
                        }
                        (2, false) => {
                            set_selection_start.set(Some((x.floor() as i64, y.floor() as i64)));
                            set_selection_end.set(Some((x.floor() as i64, y.floor() as i64)));
                            set_is_selection_menu_shown.set(false);
                            ev.prevent_default();
                        }
                        _ => {}
                    }
                }

                on:mousemove=move |ev| {
                    let (x, y) = offset_to_world(ev.offset_x(), ev.offset_y());
                    if let Some((px, py)) = pan.get_value() {
                        viewport
                            .update(|vp| {
                                vp.origin.0 += px - x;
                                vp.origin.1 += py - y;
                            });
                    } else {
                        cursor.set((x, y));
                    }
                    if selection_start.get().is_some() && (ev.buttons() & 2) != 0 {
                        set_selection_end.set(Some((x.floor() as i64, y.floor() as i64)));
                    }
                }

                on:mouseup=move |ev| {
                    if (ev.buttons() & 0b101) == 0 {
                        pan.set_value(None);
                    }
                    match ev.button() {
                        2 => {
                            set_is_selection_menu_shown.set(true);
                        }
                        _ => {}
                    }
                }

                on:wheel=move |ev| {
                    let (x, y) = offset_to_world(ev.offset_x(), ev.offset_y());
                    let factor = std::f64::consts::E
                        .powf(-ev.delta_y() * (if ev.ctrl_key() { 10.0 } else { 1.0 }) / 1000.0);
                    viewport
                        .update(|vp| {
                            vp.zoom_at(factor, x, y);
                        });
                    ev.prevent_default();
                }

                on:keydown=move |ev| {
                    keys.update_value(|ks| {
                        if !ks.contains(&ev.key()) {
                            ks.push(ev.key())
                        }
                    });
                    match (ev.key().as_str(), ev.ctrl_key()) {
                        ("a", true) => {
                            if universe.with(|u| u.population()) > 0 {
                                let (x1, y1, x2, y2) = universe.with(|u| u.get_bounding_rect());
                                set_selection_start.set(Some((x1, y1)));
                                set_selection_end.set(Some((x2, y2)));
                                set_is_selection_menu_shown.set(true);
                            }
                            ev.prevent_default();
                        }
                        ("c", true) => {
                            copy_selection_rle();
                        }
                        ("v", true) => {
                            if let Some(rle) = text.get() {
                                set_selection_start.set(None);
                                set_selection_end.set(None);
                                start_pasting(&rle);
                            }
                        }
                        ("z", true) => {
                            universe.update(|u| u.undo());
                        }
                        ("Z", true) | ("y", true) => {
                            universe.update(|u| u.redo());
                        }
                        ("Delete", _) => {
                            if let Some((x1, y1, x2, y2)) = selection_rect.get() {
                                universe
                                    .update(|u| {
                                        u.clear_rect(x1, y1, x2, y2);
                                    });
                            }
                        }
                        ("Escape", _) => {
                            set_selection_start.set(None);
                            set_selection_end.set(None);
                        }
                        _ => {}
                    }
                }
                on:keyup=move |ev| {
                    keys.update_value(|ks| ks.retain(|k| *k != ev.key()));
                    match ev.key().as_str() {
                        " " => {
                            if !did_pan.get_value() {
                                is_ticking.update(|x| *x = !*x);
                            }
                        }
                        _ => {}
                    }
                }
            >

                <Stage canvas_size=canvas_size set_canvas_size=set_canvas_size>
                    <Layer draw=move |c, raf_args| {
                        let now = raf_args.timestamp;
                        if is_ticking.get() && now - prev_tick.get_value() > 1000.0 / tps.get() {
                            universe
                                .update(|u| {
                                    u.step();
                                });
                            prev_tick.set_value(now);
                        }
                        if !is_canvas_dirty.get_value() {
                            return;
                        }
                        c.clear();
                        universe
                            .with(|u| {
                                draw::draw_node(c, &viewport.get(), u, 0xFFFFFFFF);
                            });
                        is_canvas_dirty.set_value(false);
                    } />
                    // FIX: idle performance
                    <SelectionLayer />
                    <PasteLayer />
                </Stage>
            </div>
            <SelectionOverlay is_open=is_selection_menu_shown />
            <div
                on:click=|e| e.stop_propagation()
                class="flex flex-col z-10 absolute bottom-0 inset-x-0 gap-4 pointer-events-none"
            >
                <div class="relative flex justify-center">
                    <div class="absolute left-4 pointer-events-auto">
                        <AppMenu />
                    </div>
                    <Controls />
                </div>
                <Status />
            </div>
        </div>
    }
}
