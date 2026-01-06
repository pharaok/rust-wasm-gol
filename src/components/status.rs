use crate::{
    app::{GolContext, GolParams},
    components::Divider,
};
use leptos::prelude::*;
use leptos_router::hooks::use_params;

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

fn metric_string(n: f64) -> String {
    if n >= 1e15 {
        format!("{:.1}P", n / 1e15)
    } else if n >= 1e12 {
        format!("{:.1}T", n / 1e12)
    } else if n >= 1e9 {
        format!("{:.1}G", n / 1e9)
    } else if n >= 1e6 {
        format!("{:.1}M", n / 1e6)
    } else if n >= 1e3 {
        format!("{:.1}k", n / 1e3)
    } else {
        format!("{:.1}", n)
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
    let ratio = move || {
        let cell_size = canvas.with(|gc| gc.as_ref().map(|gc| gc.cell_size).unwrap_or(1.0));
        if cell_size < 1.0 {
            format!("1px:{}", metric_string(1.0 / cell_size))
        } else {
            format!("{}px:1", metric_string(cell_size))
        }
    };

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
                <Item>{move || format!("Step: {}", 1i64 << universe.with(|u| u.step))}</Item>
                <Divider />
                <Item>{move || format!("Gen: {}", universe.with(|u| u.generation))}</Item>
                <Divider />
                <Item>{move || format!("Pop: {}", universe.with(|u| u.get_population()))}</Item>
                <Divider />
                <Item on_press=Box::new(move || {
                    if universe.with(|u| u.get_population()) == 0 {
                        return;
                    }
                    set_canvas
                        .update(|gc| {
                            let gc = gc.as_mut().unwrap();
                            let (t, l, b, r) = universe.with(|u| u.get_bounding_rect());
                            gc.fit_rect(t as f64, l as f64, (r - l + 1) as f64, (b - l + 1) as f64);
                            gc.zoom_at_center(0.6);
                        });
                })>{ratio}</Item>
                <Divider />
                <Item>
                    {move || {
                        format!(
                            "{}, {}",
                            cursor.get().0.floor() as i64,
                            cursor.get().1.floor() as i64,
                        )
                    }}

                </Item>
            </div>
        </div>
    }
}
