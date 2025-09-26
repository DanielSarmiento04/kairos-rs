use leptos::*;
use leptos_router::*;

#[component]
pub fn Header() -> impl IntoView {
    let location = use_location();
    
    // Get current page title based on pathname
    let page_title = move || {
        let pathname = location.pathname.get();
        match pathname.as_str() {
            "/" => "Dashboard",
            "/routes" => "Routes",
            "/metrics" => "Metrics",  
            "/config" => "Configuration",
            "/health" => "Health",
            _ => "Kairos Gateway"
        }
    };

    view! {
        <header class="header">
            <div class="header-left">
                <h1 class="page-title">{page_title}</h1>
                <div class="breadcrumb">
                    <span class="breadcrumb-item">"🏠 Admin"</span>
                    <span class="breadcrumb-separator">"/"</span>
                    <span class="breadcrumb-item active">{page_title}</span>
                </div>
            </div>
            
            <div class="header-right">
                <div class="header-actions">
                    <div class="status-indicator">
                        <span class="status-dot healthy"></span>
                        <span class="status-text">"Gateway Online"</span>
                    </div>
                    
                    <div class="header-menu">
                        <button class="header-menu-btn">
                            <span class="user-avatar">"👤"</span>
                            <span class="user-name">"Admin"</span>
                        </button>
                    </div>
                </div>
            </div>
        </header>
    }
}