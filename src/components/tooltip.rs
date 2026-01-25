use std::sync::Arc;

use leptos::{html, prelude::*};
use leptos_use::{UseTimeoutFnReturn, use_timeout_fn};

use crate::components::{Popover, PopoverContext, PopoverPlacement, Surface};

#[derive(Clone)]
pub struct TooltipContext {
    pub start_open: Arc<dyn Fn(()) + Send + Sync>,
    pub stop_open: Arc<dyn Fn() + Send + Sync>,
    pub start_close: Arc<dyn Fn(()) + Send + Sync>,
    pub stop_close: Arc<dyn Fn() + Send + Sync>,
}

#[component]
pub fn TooltipTrigger(children: Children) -> impl IntoView {
    let (is_open, set_is_open) = signal(false);
    let trigger_ref = NodeRef::<html::Div>::new();
    provide_context(PopoverContext {
        is_open,
        set_is_open,
        trigger_ref,
    });

    let UseTimeoutFnReturn {
        start: start_open,
        stop: stop_open,
        ..
    } = use_timeout_fn(
        move |_| {
            set_is_open.set(true);
        },
        500.0,
    );
    let UseTimeoutFnReturn {
        start: start_close,
        stop: stop_close,
        ..
    } = use_timeout_fn(
        move |_| {
            set_is_open.set(false);
        },
        250.0,
    );
    let (start_open, stop_open) = (Arc::new(start_open), Arc::new(stop_open));
    let (start_close, stop_close) = (Arc::new(start_close), Arc::new(stop_close));
    provide_context(TooltipContext {
        start_open: start_open.clone(),
        stop_open: stop_open.clone(),
        start_close: start_close.clone(),
        stop_close: stop_close.clone(),
    });

    view! {
        <div
            node_ref=trigger_ref
            on:mouseenter=move |_| {
                start_open(());
            }
            on:mouseleave=move |_| {
                stop_open();
                start_close(());
            }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn Tooltip(children: ChildrenFn) -> impl IntoView {
    let TooltipContext {
        start_close,
        stop_close,
        ..
    } = use_context::<TooltipContext>().unwrap();

    let children = StoredValue::new(children);
    let start_close = StoredValue::new(start_close);
    let stop_close = StoredValue::new(stop_close);
    view! {
        <Popover placement=PopoverPlacement::Top class="p-2">
            <Surface
                class="px-2 py-1"
                on:mouseenter=move |_| {
                    stop_close.read_value()();
                }
                on:mouseleave=move |_| {
                    start_close.read_value()(());
                }
            >
                {children.read_value()()}
                <div class="absolute left-1/2 -translate-x-1/2 bg-inherit w-2 h-2 rotate-45" />
            </Surface>
        </Popover>
    }
}
