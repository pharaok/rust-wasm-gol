use leptos::*;

use crate::components::{canvas::GolContext, Divider};

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
    let zoom = move || canvas.with(|gc| gc.as_ref().map(|gc| gc.zoom()).unwrap_or(1.0));

    view! {
        <div class="text-white flex justify-end font-mono relative text-sm">
            <Item>{move || format!("Step: {}", 1 << universe.with(|u| u.step))}</Item>
            <Divider/>
            <Item>{move || format!("Gen: {}", universe.with(|u| u.generation))}</Item>
            <Divider/>
            <Item>{move || format!("Pop: {}", universe.with(|u| u.root.borrow().population))}</Item>
            <Divider/>
            <Item on_press=Box::new(move || {
                set_canvas
                    .update(|gc| {
                        let gc = gc.as_mut().unwrap();
                        gc.zoom_at(
                            1.0 / gc.zoom(),
                            gc.origin.0 + (gc.width() / 2.0),
                            gc.origin.1 + (gc.height() / 2.0),
                        );
                    });
            })>{move || format!("{:.0}%", zoom() * 100.0)}</Item>
            <Divider/>
            <Item>
                {move || {
                    format!("{}, {}", cursor().0.floor() as i32, cursor().1.floor() as i32)
                }}

            </Item>
        </div>
    }
}
