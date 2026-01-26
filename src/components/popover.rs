use leptos::portal::Portal;
use leptos::{html, prelude::*};
use tailwind_fuse::tw_merge;
use web_sys::HtmlDivElement;

#[derive(Clone, Copy)]
pub enum PopoverPlacement {
    Bottom,
    BottomStart,
    BottomEnd,
    Left,
    LeftStart,
    LeftEnd,
    Right,
    RightStart,
    RightEnd,
    Top,
    TopStart,
    TopEnd,
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
    let offset_x = match placement {
        PopoverPlacement::BottomStart
        | PopoverPlacement::Left
        | PopoverPlacement::LeftStart
        | PopoverPlacement::LeftEnd
        | PopoverPlacement::TopStart => -rect.width() / 2.0,
        PopoverPlacement::BottomEnd
        | PopoverPlacement::Right
        | PopoverPlacement::RightStart
        | PopoverPlacement::RightEnd
        | PopoverPlacement::TopEnd => rect.width() / 2.0,
        _ => 0.0,
    };
    let offset_y = match placement {
        PopoverPlacement::LeftStart
        | PopoverPlacement::Top
        | PopoverPlacement::TopStart
        | PopoverPlacement::TopEnd
        | PopoverPlacement::RightStart => -rect.height() / 2.0,
        PopoverPlacement::LeftEnd
        | PopoverPlacement::Bottom
        | PopoverPlacement::BottomStart
        | PopoverPlacement::BottomEnd
        | PopoverPlacement::RightEnd => rect.height() / 2.0,
        _ => 0.0,
    };
    (center_x + offset_x, center_y + offset_y)
}

#[component]
pub fn PopoverTrigger(
    children: Children,
    #[prop(optional, into)] is_open: Option<ReadSignal<bool>>,
    #[prop(optional, into)] set_is_open: Option<WriteSignal<bool>>,
) -> impl IntoView {
    let is_controlled = is_open.is_some();
    let (is_open, set_is_open) = if let (Some(is_open), Some(set_is_open)) = (is_open, set_is_open)
    {
        (is_open, set_is_open)
    } else {
        signal(false)
    };
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
                if !is_controlled {
                    set_is_open.set(true);
                }
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
                            "z-40 absolute",
                            class.read_value().get().to_string()
                        )
                    }
                    style:inset=move || {
                        let (x, y) = pos.get();
                        format!("{}px auto auto {}px", y, x)
                    }
                    style:transform=move || {
                        let translate_x = match placement {
                            PopoverPlacement::TopStart
                            | PopoverPlacement::Right
                            | PopoverPlacement::RightStart
                            | PopoverPlacement::RightEnd
                            | PopoverPlacement::BottomStart => "0",
                            PopoverPlacement::TopEnd
                            | PopoverPlacement::Left
                            | PopoverPlacement::LeftStart
                            | PopoverPlacement::LeftEnd
                            | PopoverPlacement::BottomEnd => "-100%",
                            _ => "-50%",
                        };
                        let translate_y = match placement {
                            PopoverPlacement::LeftStart
                            | PopoverPlacement::Bottom
                            | PopoverPlacement::BottomStart
                            | PopoverPlacement::BottomEnd
                            | PopoverPlacement::RightStart => "0",
                            PopoverPlacement::LeftEnd
                            | PopoverPlacement::Top
                            | PopoverPlacement::TopStart
                            | PopoverPlacement::TopEnd
                            | PopoverPlacement::RightEnd => "-100%",
                            _ => "-50%",
                        };
                        format!("translate({}, {})", translate_x, translate_y)
                    }
                >
                    {children.read_value()()}
                </div>
            </Show>
        </Portal>
    }
}
