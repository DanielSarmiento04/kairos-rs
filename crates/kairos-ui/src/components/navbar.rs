//! Top navigation bar component.

use leptos::prelude::*;
use leptos_router::hooks::use_location;

/// Top navigation bar with branding and connection status.
#[component]
pub fn Navbar() -> impl IntoView {
    let location = use_location();
    
    // Extract current page name from path
    let page_name = move || {
        let path = location.pathname.get();
        match path.as_str() {
            "/" => "Dashboard",
            "/routes" => "Routes",
            "/metrics" => "Metrics",
            "/config" => "Configuration",
            "/health" => "Health",
            "/profile" => "Profile",
            _ => "Kairos Gateway",
        }
    };
    
    view! {
        <nav class="navbar">
            <div class="navbar-brand">
                <h1 class="navbar-title">
                    <span class="navbar-icon">"🔄"</span>
                    " Kairos Gateway"
                </h1>
            </div>
            
            <div class="navbar-center">
                <span class="navbar-page-title">{page_name}</span>
            </div>
            
            <div class="navbar-actions">
                <div class="navbar-status">
                    <span class="status-dot status-dot-active"></span>
                    <span class="status-text">"Connected"</span>
                </div>
            </div>
        </nav>
    }
}
