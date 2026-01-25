use leptos::prelude::*;
use leptos_router::components::Outlet;

use crate::components::{PatternLibrary, Sidebar, SidebarTrigger};

#[component]
pub fn Layout() -> impl IntoView {
    view! {
        <Outlet />
        <Sidebar>
            <SidebarTrigger>PATTERNS</SidebarTrigger>
            <PatternLibrary />
        </Sidebar>
    }
}
