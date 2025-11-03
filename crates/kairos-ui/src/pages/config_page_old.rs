//! Configuration management page for gateway settings.

use leptos::prelude::*;

/// Configuration page (placeholder for now).
#[component]
pub fn ConfigPage() -> impl IntoView {
    view! {
        <div class="config-page">
            <div class="page-header">
                <h1 class="page-title">"Configuration"</h1>
                <p class="page-subtitle">"Manage gateway configuration and settings"</p>
                
                <button class="btn btn-secondary">
                    "🔄 Reload Configuration"
                </button>
            </div>
            
            <div class="content-placeholder">
                <div class="placeholder-icon">"⚙️"</div>
                <h2>"Configuration Editor Coming Soon"</h2>
                <p>"Edit and manage gateway configuration through a user-friendly interface."</p>
                <p class="placeholder-features">
                    "Features:" <br/>
                    "• Edit JWT authentication settings" <br/>
                    "• Configure rate limiting policies" <br/>
                    "• Manage CORS and security headers" <br/>
                    "• Update circuit breaker thresholds" <br/>
                    "• Hot-reload configuration changes" <br/>
                    "• Configuration validation"
                </p>
            </div>
        </div>
    }
}
