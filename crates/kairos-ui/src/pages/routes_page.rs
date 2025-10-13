//! Routes management page for viewing and editing gateway routes.

use leptos::prelude::*;

/// Routes management page (placeholder for now).
#[component]
pub fn RoutesPage() -> impl IntoView {
    view! {
        <div class="routes-page">
            <div class="page-header">
                <h1 class="page-title">"Routes Management"</h1>
                <p class="page-subtitle">"Configure and manage gateway routes"</p>
                
                <button class="btn btn-primary">
                    "➕ Add Route"
                </button>
            </div>
            
            <div class="content-placeholder">
                <div class="placeholder-icon">"🛣️"</div>
                <h2>"Routes Management Coming Soon"</h2>
                <p>"Create, edit, and delete gateway routes through an intuitive interface."</p>
                <p class="placeholder-features">
                    "Features:" <br/>
                    "• View all configured routes" <br/>
                    "• Add new routes with validation" <br/>
                    "• Edit existing route configurations" <br/>
                    "• Delete unused routes" <br/>
                    "• Test routes with live requests"
                </p>
            </div>
        </div>
    }
}
