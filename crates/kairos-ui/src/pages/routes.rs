use leptos::*;

#[component]  
pub fn RoutesPage() -> impl IntoView {
    view! {
        <div class="routes-page">
            <div class="page-header">
                <h1>"Routes Management"</h1>
                <p>"Configure and monitor your gateway routes"</p>
            </div>
            <div class="coming-soon">
                <p>"🚧 Routes management interface coming soon!"</p>
            </div>
        </div>
    }
}

#[component]
pub fn MetricsPage() -> impl IntoView {
    view! {
        <div class="metrics-page">
            <div class="page-header">
                <h1>"Metrics & Analytics"</h1>
                <p>"Monitor your gateway performance"</p>
            </div>
            <div class="coming-soon">
                <p>"📈 Advanced metrics dashboard coming soon!"</p>
            </div>
        </div>
    }
}

#[component]
pub fn ConfigPage() -> impl IntoView {
    view! {
        <div class="config-page">
            <div class="page-header">
                <h1>"Configuration"</h1>
                <p>"Edit gateway configuration"</p>
            </div>
            <div class="coming-soon">
                <p>"⚙️ Configuration editor coming soon!"</p>
            </div>
        </div>
    }
}

#[component]
pub fn HealthPage() -> impl IntoView {
    view! {
        <div class="health-page">
            <div class="page-header">
                <h1>"Health Status"</h1>
                <p>"Monitor system health and uptime"</p>
            </div>
            <div class="coming-soon">
                <p>"💚 Health monitoring dashboard coming soon!"</p>
            </div>
        </div>
    }
}