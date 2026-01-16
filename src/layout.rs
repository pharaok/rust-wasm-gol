use leptos::prelude::*;
use leptos_router::components::Outlet;

use crate::components::{Menu, MenuTrigger, PatternLibrary};

#[component]
pub fn Layout() -> impl IntoView {
    view! {
        <Outlet />
        <Menu>
            <MenuTrigger>PATTERNS</MenuTrigger>
            <PatternLibrary />
        </Menu>
    }
}
