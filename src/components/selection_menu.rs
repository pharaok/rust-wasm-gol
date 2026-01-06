use leptos::prelude::*;

use crate::components::{Button, ButtonVariant, Divider, Icon};

#[component]
pub fn SelectionMenu() -> impl IntoView {
    view! {
        <div class="rounded-lg pointer-events-auto flex overflow-hidden">
            <Button
                variant=ButtonVariant::Icon
                disabled=true
                on_press=move || {
                    unimplemented!("");
                }
            >

                <Icon icon=icondata::LuDice5 />
            </Button>
            <Divider />
            <Button
                variant=ButtonVariant::Icon
                disabled=true
                on_press=move || {
                    unimplemented!("");
                }
            >
                <Icon icon=icondata::LuCopy />
            </Button>
            <Divider />
            <Button
                variant=ButtonVariant::Icon
                disabled=true
                on_press=move || {
                    unimplemented!("");
                }
            >
                <Icon icon=icondata::LuSave />
            </Button>
        </div>
    }
}
