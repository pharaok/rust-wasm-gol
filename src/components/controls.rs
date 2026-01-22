use crate::{
    app::GolContext,
    components::{Button, ButtonVariant, Divider, Icon},
};
use leptos::prelude::*;

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
        <div class="rounded-lg pointer-events-auto flex overflow-hidden bg-neutral-900">
            <Button
                variant=ButtonVariant::Icon
                disabled=Signal::derive_local(move || universe.with(|u| !u.can_undo()))
                on_press=move || {
                    set_universe
                        .update(|u| {
                            u.undo();
                        });
                }
            >
                <Icon icon=icondata::LuUndo2 />
            </Button>
            // <Divider />
            <Button
                variant=ButtonVariant::Icon
                disabled=Signal::derive_local(move || universe.with(|u| !u.can_redo()))
                on_press=move || {
                    set_universe
                        .update(|u| {
                            u.redo();
                        });
                }
            >
                <Icon icon=icondata::LuRedo2 />
            </Button>
            <Divider />
            <Button
                variant=ButtonVariant::Icon
                disabled=Signal::derive_local(move || universe.with(|u| u.step <= 0))
                on_press=move || { set_universe.update(|u| { u.step = (u.step - 1).max(0) }) }
            >
                <Icon icon=icondata::LuRewind />
            </Button>
            // <Divider />
            <Button
                variant=ButtonVariant::Icon
                on_press=move || {
                    set_is_ticking.update(|b| *b = !*b);
                }
            >
                {move || {
                    if is_ticking.get() {
                        view! { <Icon icon=icondata::LuPause /> }
                    } else {
                        view! { <Icon icon=icondata::LuPlay /> }
                    }
                }}
            </Button>
            // <Divider />
            <Button
                variant=ButtonVariant::Icon
                on_press=move || { set_universe.update(|u| { u.step() }) }
            >
                <Icon icon=icondata::LuStepForward />
            </Button>
            // <Divider />
            <Button
                variant=ButtonVariant::Icon
                disabled=Signal::derive_local(move || {
                    universe.with(|u| u.step >= u.level() as i32 - 2)
                })
                on_press=move || {
                    set_universe.update(|u| { u.step = (u.step + 1).min(u.level() as i32 - 2) })
                }
            >
                <Icon icon=icondata::LuFastForward />
            </Button>
        </div>
    }
}
