use crate::{
    components::{Controls, Layer, SelectionMenu, SelectionOverlay, Stage, Status, use_toast},
    draw::{self, Viewport},
    parse::rle::{self, PatternMetadata},
    universe::{InsertMode, Universe},
};
use gloo_net::http::Request;
use leptos::{ev::mousedown, html, prelude::*};
use leptos_router::hooks::*;
use leptos_router::params::Params;
use leptos_use::{UseClipboardReturn, use_clipboard, use_document, use_event_listener};

#[derive(Params, PartialEq)]
pub struct GolParams {
    pub name: Option<String>,
}

#[derive(Clone)]
pub struct GolContext {
    pub universe: ReadSignal<Universe, LocalStorage>,
    pub set_universe: WriteSignal<Universe, LocalStorage>,
    pub canvas_size: ReadSignal<(u32, u32), LocalStorage>,
    pub viewport: ReadSignal<Viewport, LocalStorage>,
    pub set_viewport: WriteSignal<Viewport, LocalStorage>,
    pub cursor: ReadSignal<(f64, f64), LocalStorage>,
    pub set_cursor: WriteSignal<(f64, f64), LocalStorage>,
    pub selection_rect: Signal<Option<(i64, i64, i64, i64)>, LocalStorage>,
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
        signal_local(Universe::with_size_and_arena_capacity(60, 1 << 16));
    let (canvas_size, set_canvas_size) = signal_local((0, 0));
    let (viewport, set_viewport) = signal_local(Viewport::new());
    let (cursor, set_cursor) = signal_local((0.0, 0.0));
    let (is_ticking, set_is_ticking) = signal_local(false);
    let tps = StoredValue::new(20.0);
    let offset_to_world = move |x: i32, y: i32| viewport.with(|vp| vp.to_world_coords(x, y));
    let pan = StoredValue::<Option<(f64, f64)>>::new(None);

    let (selection_start, set_selection_start) = signal_local::<Option<(i64, i64)>>(None);
    let (selection_end, set_selection_end) = signal_local::<Option<(i64, i64)>>(None);
    let (is_selection_menu_shown, set_is_selection_menu_shown) = signal_local(false);
    let selection_rect = Signal::derive_local(move || {
        let ((sx, sy), (ex, ey)) = (selection_start.get()?, selection_end.get()?);
        Some((sx.min(ex), sy.min(ey), sx.max(ex), sy.max(ey)))
    });

    provide_context(GolContext {
        universe,
        set_universe,
        canvas_size,
        viewport,
        set_viewport,
        cursor,
        set_cursor,
        selection_rect,
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

    let did_fit = StoredValue::new_local(false);
    Effect::new(move |_| {
        pattern_rle.track();
        did_fit.set_value(false);
    });
    Effect::new(move |_| {
        let (canvas_width, canvas_height) = canvas_size.get();
        if did_fit.get_value() || canvas_width == 0 || canvas_height == 0 {
            return;
        }
        // pattern_rle will never actually be Some(Err) because
        // the server will always return 200 OK since this is a SPA
        if let Some(Ok(rle)) = pattern_rle.get()
            && rle::parse_metadata(&rle, "", "").is_ok()
        {
            set_universe.update(|u| {
                u.clear();
                if meta {
                    if let Some(Ok(on_rle)) = meta_on_rle.unwrap().get()
                        && let Some(Ok(off_rle)) = meta_off_rle.unwrap().get()
                    {
                        let rect = rle::to_grid(&rle).unwrap();
                        u.set_grid_meta(&rect, &on_rle, &off_rle);
                    }
                } else {
                    let points = rle::iter_alive(&rle).unwrap().collect::<Vec<_>>();
                    let half = 1i64 << (u.get_level() - 1);
                    u.set_points(&points, -half, -half, half - 1, half - 1, &InsertMode::Copy);
                }
            });

            if universe.with_untracked(|u| u.get_population()) != 0 {
                let (x1, y1, x2, y2) = universe.with_untracked(|u| u.get_bounding_rect());
                set_viewport.update(|vp| {
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
        did_fit.set_value(true);
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
        prev_tick.set_value(0.0);
    });

    let keys = StoredValue::<Vec<String>, LocalStorage>::new_local(Vec::new());
    let did_pan = StoredValue::new_local(false);

    let UseClipboardReturn { copy, text, .. } = use_clipboard();
    let (paste_universe, set_paste_universe) =
        signal_local(Universe::with_size_and_arena_capacity(30, 0));
    let (paste_size, set_paste_size) = signal_local((0, 0));
    let (is_pasting, set_is_pasting) = signal_local(false);
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

    let div_ref = NodeRef::<html::Div>::new();
    div_ref.on_load(|div_el| {
        let _ = div_el.focus();
    });
    let _ = use_event_listener(use_document(), mousedown, move |ev| {
        let el = event_target::<web_sys::HtmlDivElement>(&ev);
        let tag = el.tag_name();

        if tag != "INPUT" {
            ev.prevent_default();
        }
    });

    let push_toast = use_toast();

    view! {
        <div class="absolute inset-0 w-screen h-screen overflow-hidden">
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
                            if is_pasting.get() {
                                if let Ok(points) = rle::iter_alive(&paste_rle.get_value()) {
                                    let (cx, cy) = cursor
                                        .with(|(x, y)| (x.floor() as i64, y.floor() as i64));
                                    let (width, height) = paste_size.get();
                                    set_universe
                                        .update(|u| {
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
                                set_is_pasting.set(false);
                                return;
                            }
                            if viewport.get().cell_size >= 5.0 {
                                set_universe
                                    .update(|u| {
                                        let (x, y) = (x.floor() as i64, y.floor() as i64);
                                        let v = u.get(x, y);
                                        u.set(x, y, v ^ 1);
                                    });
                            }
                            set_selection_start.set(None);
                            set_selection_end.set(None);
                            set_is_selection_menu_shown.set(false);
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
                        set_viewport
                            .update(|vp| {
                                vp.origin.0 += px - x;
                                vp.origin.1 += py - y;
                            });
                    } else {
                        set_cursor.set((x, y));
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
                    set_viewport
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
                    match ev.key().as_str() {
                        "a" => {
                            if ev.ctrl_key() {
                                let (x1, y1, x2, y2) = universe.with(|u| u.get_bounding_rect());
                                set_selection_start.set(Some((x1, y1)));
                                set_selection_end.set(Some((x2, y2)));
                                set_is_selection_menu_shown.set(true);
                                ev.prevent_default();
                            }
                        }
                        "c" => {
                            if let Some((x1, y1, x2, y2)) = selection_rect.get() {
                                universe
                                    .with(|u| {
                                        let rle = rle::from_iter(
                                            u.iter_alive_in_rect(x1, y1, x2, y2),
                                            x1,
                                            y1,
                                            x2,
                                            y2,
                                        );
                                        copy(&rle);
                                        push_toast.run("Copied RLE to clipboard!".to_owned());
                                    });
                            }
                        }
                        "v" => {
                            if let Some(rle) = text.get() {
                                set_selection_start.set(None);
                                set_selection_end.set(None);
                                set_is_pasting.set(true);
                                if rle == paste_rle.get_value() {
                                    return;
                                }
                                paste_rle.set_value(rle.clone());
                                if let Ok(points) = rle::iter_alive(&rle) {
                                    set_paste_universe
                                        .update(|u| {
                                            let half = 1i64 << (u.get_level() - 1);
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
                                if let Ok((PatternMetadata { width, height, .. }, _)) = rle::parse_metadata(
                                    &rle,
                                    "",
                                    "",
                                ) {
                                    set_paste_size.set((width as i64, height as i64));
                                }
                            }
                        }
                        "Delete" => {
                            if let Some((x1, y1, x2, y2)) = selection_rect.get() {
                                set_universe
                                    .update(|u| {
                                        u.clear_rect(x1, y1, x2, y2);
                                    });
                            }
                        }
                        "Escape" => {
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
                                set_is_ticking.update(|x| *x = !*x);
                            }
                        }
                        _ => {}
                    }
                }
            >

                <Stage canvas_size=canvas_size set_canvas_size=set_canvas_size>
                    <Layer draw=move |c, raf_args| {
                        let now = raf_args.timestamp;
                        if is_ticking.get()
                            && now - prev_tick.get_value() > 1000.0 / tps.get_value()
                        {
                            set_universe
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
                    <SelectionOverlay />
                    <Layer draw=move |c, _raf_args| {
                        if !is_paste_canvas_dirty.get_value() {
                            return;
                        }
                        if !is_pasting.get() {
                            c.clear();
                            c.draw();
                            return;
                        }
                        let (width, height) = paste_size.get();
                        let mut vp = viewport.get();
                        vp.origin.0 -= cursor.get().0.floor();
                        vp.origin.1 -= cursor.get().1.floor();
                        c.clear();
                        c.fill_rect_with_viewport(
                            &vp,
                            0.0,
                            0.0,
                            width as f64,
                            height as f64,
                            0x00FFFF3F,
                        );
                        paste_universe
                            .with(|u| {
                                draw::draw_node(c, &vp, u, 0x00FFFFBF);
                            });
                        is_paste_canvas_dirty.set_value(false);
                    } />
                </Stage>
            </div>
            <div
                class="z-10 absolute flex justify-end items-start pointer-events-none"
                style:inset=move || {
                    let vp = viewport.get();
                    let (width, _) = canvas_size.get();
                    if let Some((_, _, x2, y2)) = selection_rect.get() {
                        let (r, b) = vp.to_canvas_coords((x2 + 1) as f64, (y2 + 1) as f64);
                        format!("{}px {}px -9999px -9999px", b + 16, (width as i32) - r)
                    } else {
                        "9999px".to_owned()
                    }
                }
            >
                <Show when=is_selection_menu_shown fallback=|| view! {}>
                    <SelectionMenu />
                </Show>
            </div>
            <div on:click=|e| e.stop_propagation()>
                <div class="z-10 absolute bottom-8 left-[50%] -translate-x-[50%]">
                    <Controls />
                </div>
                <div class="absolute bottom-0 inset-x-0">
                    <Status />
                </div>
            </div>
        </div>
    }
}
