use crate::{
    app::GolContext,
    components::{Button, ButtonVariant, Divider, Icon},
};
use leptos::prelude::*;
use rand::Rng;

#[component]
pub fn SelectionMenu() -> impl IntoView {
    let GolContext {
        universe,
        set_universe,
        selection_start,
        selection_end,
        ..
    } = use_context::<GolContext>().unwrap();
    view! {
        <div class="rounded-lg pointer-events-auto flex overflow-hidden">
            <Button
                variant=ButtonVariant::Icon
                on_press=move || {
                    let mut rng = rand::rng();
                    if let Some((mut l, mut t)) = selection_start.get()
                        && let Some((mut r, mut b)) = selection_end.get()
                    {
                        (l, r) = if l < r { (l, r) } else { (r, l) };
                        (t, b) = if t < b { (t, b) } else { (b, t) };
                        set_universe
                            .update(|u| {
                                for y in t..=b {
                                    for x in l..=r {
                                        u.set(x, y, rng.random_bool(0.5) as u8);
                                    }
                                }
                            });
                    }
                }
            >

                <Icon icon=icondata::LuDice5 />
            </Button>
            <Divider />
            <Button
                variant=ButtonVariant::Icon
                on_press=move || {
                    if let Some((mut l, mut t)) = selection_start.get()
                        && let Some((mut r, mut b)) = selection_end.get()
                    {
                        (l, r) = if l < r { (l, r) } else { (r, l) };
                        (t, b) = if t < b { (t, b) } else { (b, t) };
                        set_universe
                            .update(|u| {
                                u.clear_rect(l, t, r, b);
                            });
                    }
                }
            >

                <Icon icon=icondata::LuTrash />
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
