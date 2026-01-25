use crate::{
    app::GolContext,
    components::{Button, ButtonVariant, Divider, Icon, Surface, Tooltip, TooltipTrigger},
};
use leptos::prelude::*;

#[component]
pub fn Controls() -> impl IntoView {
    let GolContext {
        universe,
        is_ticking,
        tps,
        ..
    } = use_context::<GolContext>().unwrap();

    view! {
        <Surface class="pointer-events-auto flex overflow-hidden">
            <TooltipTrigger>
                <Button
                    variant=ButtonVariant::Icon
                    disabled=Signal::derive_local(move || universe.with(|u| !u.can_undo()))
                    on_press=move || {
                        universe
                            .update(|u| {
                                u.undo();
                            });
                    }
                >
                    <Icon icon=icondata::LuUndo2 />
                </Button>
                <Tooltip>Undo</Tooltip>
            </TooltipTrigger>
            <TooltipTrigger>
                <Button
                    variant=ButtonVariant::Icon
                    disabled=Signal::derive_local(move || universe.with(|u| !u.can_redo()))
                    on_press=move || {
                        universe
                            .update(|u| {
                                u.redo();
                            });
                    }
                >
                    <Icon icon=icondata::LuRedo2 />
                </Button>
                <Tooltip>Redo</Tooltip>
            </TooltipTrigger>
            <Divider />
            <TooltipTrigger>
                <Button
                    variant=ButtonVariant::Icon
                    on_press=move || {
                        if universe.with(|u| u.step <= 0) {
                            tps.update(|tps| *tps /= 2.0);
                        } else {
                            universe.update(|u| { u.step = (u.step - 1).max(0) })
                        }
                    }
                >
                    <Icon icon=icondata::LuRewind />
                </Button>
                <Tooltip>Decrease Speed</Tooltip>
            </TooltipTrigger>
            <TooltipTrigger>
                <Button
                    variant=ButtonVariant::Icon
                    on_press=move || {
                        is_ticking.update(|b| *b = !*b);
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
                <Tooltip>{move || if is_ticking.get() { "Pause" } else { "Play" }}</Tooltip>
            </TooltipTrigger>
            <TooltipTrigger>
                <Button
                    variant=ButtonVariant::Icon
                    on_press=move || {
                        universe
                            .update(|u| {
                                u.push_snapshot();
                                u.step();
                            })
                    }
                >
                    <Icon icon=icondata::LuStepForward />
                </Button>
                <Tooltip>Step</Tooltip>
            </TooltipTrigger>
            <TooltipTrigger>
                <Button
                    variant=ButtonVariant::Icon
                    disabled=Signal::derive_local(move || {
                        universe.with(|u| u.step >= u.level() as i32 - 2)
                    })
                    on_press=move || {
                        if tps.get() < 16.0 {
                            tps.update(|tps| *tps *= 2.0);
                        } else {
                            universe.update(|u| { u.step = (u.step + 1).min(u.level() as i32 - 2) })
                        }
                    }
                >
                    <Icon icon=icondata::LuFastForward />
                </Button>
                <Tooltip>Increase Speed</Tooltip>
            </TooltipTrigger>
        </Surface>
    }
}
