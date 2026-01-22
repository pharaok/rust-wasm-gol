use crate::{
    app::GolContext,
    components::{Button, ButtonVariant, Divider, Icon, use_toast},
    parse::rle,
};
use leptos::prelude::*;
use leptos_use::{UseClipboardReturn, use_clipboard};
use rand::Rng;

#[component]
pub fn SelectionMenu() -> impl IntoView {
    let GolContext {
        universe,
        selection_rect,
        ..
    } = use_context::<GolContext>().unwrap();

    let UseClipboardReturn { copy, .. } = use_clipboard();
    let push_toast = use_toast();
    view! {
        <div class="bg-neutral-900 rounded-lg pointer-events-auto flex overflow-hidden">
            <Button
                variant=ButtonVariant::Icon
                on_press=move || {
                    if let Some((x1, y1, x2, y2)) = selection_rect.get() {
                        let mut rng = rand::rng();
                        universe
                            .update(|u| {
                                for y in y1..=y2 {
                                    for x in x1..=x2 {
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
                    if let Some((x1, y1, x2, y2)) = selection_rect.get() {
                        universe
                            .update(|u| {
                                u.clear_rect(x1, y1, x2, y2);
                            });
                    }
                }
            >

                <Icon icon=icondata::LuTrash />
            </Button>
            <Divider />
            <Button
                variant=ButtonVariant::Icon
                on_press=move || {
                    if let Some((x1, y1, x2, y2)) = selection_rect.get() {
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
                                push_toast.run("Copied RLE to clipboard!".to_owned());
                            });
                    }
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
