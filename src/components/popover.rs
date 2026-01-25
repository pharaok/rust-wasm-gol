use leptos::portal::Portal;
use leptos::{html, prelude::*};
use tailwind_fuse::tw_merge;
use web_sys::HtmlDivElement;

#[derive(Clone, Copy)]
pub enum PopoverPlacement {
    Bottom,
    Left,
    Right,
    Top,
}

#[derive(Clone)]
pub struct PopoverContext {
    pub is_open: ReadSignal<bool>,
    pub set_is_open: WriteSignal<bool>,
    pub trigger_ref: NodeRef<html::Div>,
}

pub fn popover_pos(trigger_el: HtmlDivElement, placement: PopoverPlacement) -> (f64, f64) {
    let rect = trigger_el.get_bounding_client_rect();
    let (center_x, center_y) = (
        rect.left() + rect.width() / 2.0,
        rect.top() + rect.height() / 2.0,
    );
    let (offset_x, offset_y) = match placement {
        PopoverPlacement::Bottom => (0.0, rect.height() / 2.0),
        PopoverPlacement::Left => (-rect.width() / 2.0, 0.0),
        PopoverPlacement::Right => (rect.width() / 2.0, 0.0),
        PopoverPlacement::Top => (0.0, -rect.height() / 2.0),
    };
    (center_x + offset_x, center_y + offset_y)
}

#[component]
pub fn PopoverTrigger(children: Children) -> impl IntoView {
    let (is_open, set_is_open) = signal(false);
    let trigger_ref = NodeRef::<html::Div>::new();
    provide_context(PopoverContext {
        is_open,
        set_is_open,
        trigger_ref,
    });

    view! {
        <div
            node_ref=trigger_ref
            on:click=move |_| {
                set_is_open.set(true);
            }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn Popover(
    children: ChildrenFn,
    #[prop(into, optional)] class: TextProp,
    #[prop(default=PopoverPlacement::Bottom)] placement: PopoverPlacement,
) -> impl IntoView {
    let PopoverContext {
        is_open,
        trigger_ref,
        ..
    } = use_context::<PopoverContext>().unwrap();
    let (pos, set_pos) = signal((0.0, 0.0));
    Effect::new(move |_| {
        if is_open.get() {
            set_pos.set(popover_pos(trigger_ref.get().unwrap(), placement));
        }
    });

    let class = StoredValue::new(class);
    let children = StoredValue::new(children);
    view! {
        <Portal mount=document().body().unwrap()>
            <Show when=move || is_open.get()>
                <div
                    class=move || {
                        tw_merge!(
                            "z-50 absolute",
                            match placement {
                                PopoverPlacement::Bottom => "-translate-x-1/2",
                                PopoverPlacement::Left=> "-translate-x-full -translate-y-1/2",
                                PopoverPlacement::Right => "-translate-y-1/2",
                                PopoverPlacement::Top => "-translate-x-1/2 -translate-y-full",
                            },
                            class.read_value().get().to_string()
                        )
                    }
                    style:inset=move || {
                        let (x, y) = pos.get();
                        format!("{}px auto auto {}px", y, x)
                    }
                >
                    {children.read_value()()}
                </div>
            </Show>
        </Portal>
    }
}
