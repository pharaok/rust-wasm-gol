use leptos::prelude::*;
use leptos_router::hooks::use_params;

use crate::{
    app::{GolContext, GolParams},
    components::Divider,
};

#[component]
pub fn Item(
    children: Children,
    #[prop(optional)] on_press: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    view! {
        <span
            class=format!(
                "px-4 pb-1 {}",
                if on_press.is_some() { "cursor-pointer hover:bg-white/10" } else { "" },
            )

            on:click=move |_| {
                if let Some(on_press) = &on_press {
                    on_press()
                }
            }
        >

            {children()}
        </span>
    }
}

#[component]
pub fn Status() -> impl IntoView {
    let GolContext {
        universe,
        cursor,
        canvas,
        set_canvas,
        ..
    } = use_context::<GolContext>().unwrap();
    let zoom = move || canvas.with(|gc| gc.as_ref().map(|gc| gc.get_zoom()).unwrap_or(1.0));

    let params = use_params::<GolParams>();
    let pattern_name = move || {
        params.with(|p| {
            p.as_ref()
                .map(|p| p.name.clone().unwrap_or_default())
                .unwrap_or_default()
        })
    };

    view! {
        <div class="flex justify-between text-white text-sm font-mono">
            <div>
                <Item>{move || pattern_name}</Item>
            </div>
            <div class="inline-flex">
                <Item>{move || format!("Step: {}", 1 << universe.with(|u| u.step))}</Item>
                <Divider />
                <Item>{move || format!("Gen: {}", universe.with(|u| u.generation))}</Item>
                <Divider />
                <Item>{move || format!("Pop: {}", universe.with(|u| u.get_population()))}</Item>
                <Divider />
                <Item on_press=Box::new(move || {
                    set_canvas
                        .update(|gc| {
                            let gc = gc.as_mut().unwrap();
                            gc.zoom_at_center(1.0 / gc.get_zoom());
                        });
                })>{move || format!("{:.0}%", zoom() * 100.0)}</Item>
                <Divider />
                <Item>
                    {move || {
                        format!(
                            "{}, {}",
                            cursor.get().0.floor() as i32,
                            cursor.get().1.floor() as i32,
                        )
                    }}

                </Item>
            </div>
        </div>
    }
}
