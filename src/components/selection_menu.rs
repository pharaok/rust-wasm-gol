use crate::{
    app::GolContext,
    components::{Button, ButtonVariant, Divider, Icon},
    parse::rle,
};
use leptos::prelude::*;
use leptos_use::{UseClipboardReturn, use_clipboard};
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
    let selection_rect = move || {
        let (mut x1, mut y1) = selection_start.get().unwrap();
        let (mut x2, mut y2) = selection_end.get().unwrap();
        (x1, x2) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        (y1, y2) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        (x1, y1, x2, y2)
    };

    let UseClipboardReturn { copy, .. } = use_clipboard();
    view! {
        <div class="rounded-lg pointer-events-auto flex overflow-hidden">
            <Button
                variant=ButtonVariant::Icon
                on_press=move || {
                    let mut rng = rand::rng();
                    let (x1, y1, x2, y2) = selection_rect();
                    set_universe
                        .update(|u| {
                            for y in y1..=y2 {
                                for x in x1..=x2 {
                                    u.set(x, y, rng.random_bool(0.5) as u8);
                                }
                            }
                        });
                }
            >

                <Icon icon=icondata::LuDice5 />
            </Button>
            <Divider />
            <Button
                variant=ButtonVariant::Icon
                on_press=move || {
                    let (x1, y1, x2, y2) = selection_rect();
                    set_universe
                        .update(|u| {
                            u.clear_rect(x1, y1, x2, y2);
                        });
                }
            >

                <Icon icon=icondata::LuTrash />
            </Button>
            <Divider />
            <Button
                variant=ButtonVariant::Icon
                on_press=move || {
                    let (x1, y1, x2, y2) = selection_rect();
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
                        });
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
