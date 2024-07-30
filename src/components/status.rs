use leptos::*;

use crate::components::canvas::GolContext;

#[component]
pub fn Divider() -> impl IntoView {
    view! { <div class="border-l-2 border-white/20"></div> }
}

#[component]
pub fn Item(children: Children) -> impl IntoView {
    view! { <span class="px-4 pb-1">{children()}</span> }
}

#[component]
pub fn Status() -> impl IntoView {
    let GolContext {
        universe,
        cursor,
        step,
        ..
    } = use_context::<GolContext>().unwrap();

    view! {
        <div class="text-white flex font-mono relative text-sm">
            <Item>{move || format!("Step: {}", 1 << step())}</Item>
            <Divider/>
            <Item>{move || format!("Gen: {}", universe().generation)}</Item>
            <Divider/>
            <Item>{move || format!("Pop: {}", universe().root.borrow().population.get())}</Item>
            <Divider/>
            <Item>
                {move || {
                    format!("{}, {}", cursor().0.floor() as i32, cursor().1.floor() as i32)
                }}

            </Item>
        </div>
    }
}
