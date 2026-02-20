//! Left sidebar navigation component.

use leptos::prelude::*;
use leptos_router::{hooks::use_location, components::A};

/// Left sidebar with navigation links.
#[component]
pub fn Sidebar() -> impl IntoView {
    let location = use_location();
    
    // Helper to determine if a path is active
    let is_active = move |path: &str| {
        location.pathname.get() == path
    };
    
    view! {
        <aside class="sidebar">
            <nav class="sidebar-nav">
                <A href="/" class:active=move || is_active("/") attr:class="sidebar-link">
                    <span class="sidebar-icon">"📊"</span>
                    <span class="sidebar-text">"Dashboard"</span>
                </A>
                
                <A href="/routes" class:active=move || is_active("/routes") attr:class="sidebar-link">
                    <span class="sidebar-icon">"🛣️"</span>
                    <span class="sidebar-text">"Routes"</span>
                </A>
                
                <A href="/metrics" class:active=move || is_active("/metrics") attr:class="sidebar-link">
                    <span class="sidebar-icon">"📈"</span>
                    <span class="sidebar-text">"Metrics"</span>
                </A>
                
                <A href="/config" class:active=move || is_active("/config") attr:class="sidebar-link">
                    <span class="sidebar-icon">"⚙️"</span>
                    <span class="sidebar-text">"Configuration"</span>
                </A>
                
                <A href="/health" class:active=move || is_active("/health") attr:class="sidebar-link">
                    <span class="sidebar-icon">"❤️"</span>
                    <span class="sidebar-text">"Health"</span>
                </A>
                
                <A href="/profile" class:active=move || is_active("/profile") attr:class="sidebar-link">
                    <span class="sidebar-icon">"👤"</span>
                    <span class="sidebar-text">"Profile"</span>
                </A>
            </nav>
            
            <div class="sidebar-footer">
                <div class="sidebar-version">
                    <span class="version-label">"Version"</span>
                    <span class="version-number">"0.1.0"</span>
                </div>
            </div>
        </aside>
    }
}
