//! Advanced metrics visualization page.

use leptos::prelude::*;

/// Metrics page (placeholder for now).
#[component]
pub fn MetricsPage() -> impl IntoView {
    view! {
        <div class="metrics-page">
            <div class="page-header">
                <h1 class="page-title">"Metrics & Analytics"</h1>
                <p class="page-subtitle">"Detailed performance metrics and analytics"</p>
            </div>
            
            <div class="content-placeholder">
                <div class="placeholder-icon">"📈"</div>
                <h2>"Advanced Metrics Coming Soon"</h2>
                <p>"Deep dive into gateway performance with detailed charts and analytics."</p>
                <p class="placeholder-features">
                    "Features:" <br/>
                    "• Historical metrics with time-series charts" <br/>
                    "• Per-route performance breakdown" <br/>
                    "• Error rate analysis and trends" <br/>
                    "• Circuit breaker state history" <br/>
                    "• Custom metric queries" <br/>
                    "• Export metrics data"
                </p>
            </div>
        </div>
    }
}
