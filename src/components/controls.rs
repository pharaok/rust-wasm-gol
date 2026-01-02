use leptos::prelude::*;

use crate::{
    app::GolContext,
    components::{Button, ButtonVariant, Divider, Icon},
};

#[component]
pub fn Controls() -> impl IntoView {
    let GolContext {
        universe,
        set_universe,
        is_ticking,
        set_is_ticking,
        ..
    } = use_context::<GolContext>().unwrap();

    view! {
        <div class="rounded-lg pointer-events-auto flex overflow-hidden">
            <Button
                variant=ButtonVariant::Icon
                disabled=Signal::derive(move || universe.with(|u| u.step <= 0))
                on_press=move || { set_universe.update(|u| { u.step = (u.step - 1).max(0) }) }
            >
                <Icon icon=icondata::LuRewind />
            </Button>
            <Divider />
            <Button
                variant=ButtonVariant::Icon
                disabled=true
                on_press=move || {
                    unimplemented!("history");
                }
            >

                <Icon icon=icondata::LuStepBack />
            </Button>
            <Divider />
            <Button
                variant=ButtonVariant::Icon
                on_press=move || {
                    set_is_ticking.update(|b| *b = !*b);
                }
            >

                {move || {
                    if is_ticking() {
                        view! { <Icon icon=icondata::LuPause /> }
                    } else {
                        view! { <Icon icon=icondata::LuPlay /> }
                    }
                }}

            </Button>
            <Divider />
            <Button
                variant=ButtonVariant::Icon
                on_press=move || { set_universe.update(|u| { u.step() }) }
            >
                <Icon icon=icondata::LuStepForward />
            </Button>
            <Divider />
            <Button
                variant=ButtonVariant::Icon
                disabled=Signal::derive(move || {
                    universe.with(|u| u.step >= u.get_level() as i32 - 2)
                })

                on_press=move || {
                    set_universe.update(|u| { u.step = (u.step + 1).min(u.get_level() as i32 - 2) })
                }
            >

                <Icon icon=icondata::LuFastForward />
            </Button>
        </div>
    }
}
