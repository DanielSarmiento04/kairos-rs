use leptos::*;use leptos::*;

use leptos::logging::log;use leptos::logging::log;

use kairos_client::{GatewayClient, ClientError, MetricsSnapshot, HealthStatus};// TODO: Add kairos_client when ready for WASM

use wasm_bindgen_futures::spawn_local;// use kairos_client::{GatewayClient, ClientError, MetricsSnapshot, HealthStatus};

use wasm_bindgen_futures::spawn_local;

#[derive(Debug, Clone)]

pub struct DashboardMetrics {/* 

    pub requests_per_sec: u64,TODO: Temporarily disabled - will be re-enabled when kairos_client supports WASM compilation

    pub success_rate: f64,

    pub avg_latency: f64,#[derive(Debug, Clone)]

    pub active_routes: u64,pub struct DashboardMetrics {

    pub active_connections: u64,    pub requests_per_sec: u64,

    pub uptime: String,    pub success_rate: f64,

}    pub avg_latency: f64,

    pub active_routes: u64,

impl Default for DashboardMetrics {    pub active_connections: u64,

    fn default() -> Self {    pub uptime: String,

        Self {}

            requests_per_sec: 0,

            success_rate: 0.0,impl Default for DashboardMetrics {

            avg_latency: 0.0,    fn default() -> Self {

            active_routes: 0,        Self {

            active_connections: 0,            requests_per_sec: 0,

            uptime: "Loading...".to_string(),            success_rate: 0.0,

        }            avg_latency: 0.0,

    }            active_routes: 0,

}            active_connections: 0,

            uptime: "Loading...".to_string(),

#[component]        }

pub fn Dashboard() -> impl IntoView {    }

    let (metrics, set_metrics) = create_signal(DashboardMetrics::default());}

    let (health_status, set_health_status) = create_signal("Loading...".to_string());

    let (loading, set_loading) = create_signal(true);#[component]

    let (error_msg, set_error_msg) = create_signal(None::<String>);pub fn Dashboard() -> impl IntoView {

    let (metrics, set_metrics) = create_signal(DashboardMetrics::default());

    // Fetch data on component mount    let (health_status, set_health_status) = create_signal("Loading...".to_string());

    create_effect(move |_| {    let (loading, set_loading) = create_signal(true);

        spawn_local(async move {    let (error_msg, set_error_msg) = create_signal(None::<String>);

            match fetch_dashboard_data().await {

                Ok((health, metrics_data)) => {    // Fetch data on component mount

                    let health_status_text = health.status.clone();    create_effect(move |_| {

                    let uptime = format_uptime(&health);        spawn_local(async move {

                                match fetch_dashboard_data().await {

                    set_health_status.set(health_status_text);                Ok((health, metrics_data)) => {

                    set_metrics.set(DashboardMetrics {                    let health_status = health.status.clone();

                        requests_per_sec: calculate_rps(&metrics_data),                    let uptime = format_uptime(&health);

                        success_rate: calculate_success_rate(&metrics_data),                    

                        avg_latency: metrics_data.average_response_time_ms,                    set_health_status.set(health_status);

                        active_routes: 5, // TODO: Get from actual data                    set_metrics.set(DashboardMetrics {

                        active_connections: metrics_data.active_connections,                        requests_per_sec: calculate_rps(&metrics_data),

                        uptime,                        success_rate: calculate_success_rate(&metrics_data),

                    });                        avg_latency: metrics_data.average_response_time_ms,

                    set_error_msg.set(None);                        active_routes: 4, // From config - TODO: make dynamic

                    set_loading.set(false);                        active_connections: metrics_data.active_connections,

                }                        uptime,

                Err(e) => {                    });

                    log!("Failed to fetch dashboard data: {:?}", e);                    set_error_msg.set(None);

                    set_error_msg.set(Some(format!("Failed to load data: {}", e)));                }

                    set_loading.set(false);                Err(e) => {

                }                    log!("Error fetching dashboard data: {:?}", e);

            }                    set_error_msg.set(Some(format!("Failed to connect to gateway: {}", e)));

        });                }

    });            }

            set_loading.set(false);

    view! {        });

        <div class="dashboard">    });

            <div class="dashboard-header">

                <h1>"🚀 Gateway Dashboard"</h1>    // Auto-refresh every 30 seconds

                <p class="dashboard-subtitle">"Real-time monitoring and metrics"</p>    let _refresh_interval = create_effect(move |_| {

                <div class="dashboard-actions">        set_interval(

                    <button class="btn btn-secondary" on:click=move |_| {            move || {

                        set_loading.set(true);                spawn_local(async move {

                        spawn_local(async move {                    if let Ok((health, metrics_data)) = fetch_dashboard_data().await {

                            match fetch_dashboard_data().await {                        let health_status = health.status.clone();

                                Ok((health, metrics_data)) => {                        let uptime = format_uptime(&health);

                                    let health_status_text = health.status.clone();                        

                                    let uptime = format_uptime(&health);                        set_health_status.set(health_status);

                                                            set_metrics.set(DashboardMetrics {

                                    set_health_status.set(health_status_text);                            requests_per_sec: calculate_rps(&metrics_data),

                                    set_metrics.set(DashboardMetrics {                            success_rate: calculate_success_rate(&metrics_data),

                                        requests_per_sec: calculate_rps(&metrics_data),                            avg_latency: metrics_data.average_response_time_ms,

                                        success_rate: calculate_success_rate(&metrics_data),                            active_routes: 4,

                                        avg_latency: metrics_data.average_response_time_ms,                            active_connections: metrics_data.active_connections,

                                        active_routes: 5,                            uptime,

                                        active_connections: metrics_data.active_connections,                        });

                                        uptime,                    }

                                    });                });

                                    set_error_msg.set(None);            },

                                }            std::time::Duration::from_secs(30),

                                Err(e) => {        );

                                    set_error_msg.set(Some(format!("Refresh failed: {}", e)));    });

                                }

                            }    view! {

                            set_loading.set(false);        <div class="dashboard">

                        });            <div class="page-header">

                    }>                <h1>"🚀 Dashboard"</h1>

                        "🔄 Refresh"                <p>"Real-time overview of your Kairos API Gateway"</p>

                    </button>                <div class="header-actions">

                </div>                    <button 

            </div>                        class="refresh-btn"

                        on:click=move |_| {

            {move || match (loading.get(), error_msg.get()) {                            set_loading.set(true);

                (true, _) => {                            spawn_local(async move {

                    view! {                                if let Ok((health, metrics_data)) = fetch_dashboard_data().await {

                        <div class="loading-container">                                    let health_status = health.status.clone();

                            <div class="loading">                                    let uptime = format_uptime(&health);

                                <div class="spinner"></div>                                    

                                <span>"Loading dashboard data..."</span>                                    set_health_status.set(health_status);

                            </div>                                    set_metrics.set(DashboardMetrics {

                        </div>                                        requests_per_sec: calculate_rps(&metrics_data),

                    }.into_view()                                        success_rate: calculate_success_rate(&metrics_data),

                }                                        avg_latency: metrics_data.average_response_time_ms,

                (false, Some(error_message)) => {                                        active_routes: 4,

                    view! {                                        active_connections: metrics_data.active_connections,

                        <div class="error-container">                                        uptime,

                            <div class="error-message">                                    });

                                <h3>"⚠️ Error Loading Data"</h3>                                    set_error_msg.set(None);

                                <p>{error_message}</p>                                }

                                <button class="btn btn-primary" on:click=move |_| {                                set_loading.set(false);

                                    set_loading.set(true);                            });

                                    set_error_msg.set(None);                        }

                                    spawn_local(async move {                    >

                                        match fetch_dashboard_data().await {                        {move || if loading.get() { "🔄 Refreshing..." } else { "🔄 Refresh" }}

                                            Ok((health, metrics_data)) => {                    </button>

                                                let health_status_text = health.status.clone();                </div>

                                                let uptime = format_uptime(&health);            </div>

                                                            

                                                set_health_status.set(health_status_text);            {move || error_msg.get().map(|err| view! {

                                                set_metrics.set(DashboardMetrics {                <div class="error-banner">

                                                    requests_per_sec: calculate_rps(&metrics_data),                    <span class="error-icon">"⚠️"</span>

                                                    success_rate: calculate_success_rate(&metrics_data),                    <span>{err}</span>

                                                    avg_latency: metrics_data.average_response_time_ms,                    <span class="error-hint">"Make sure the gateway is running on port 5900"</span>

                                                    active_routes: 5,                </div>

                                                    active_connections: metrics_data.active_connections,            })}

                                                    uptime,            

                                                });            <div class="dashboard-grid">

                                                set_error_msg.set(None);                <div class="metric-card primary">

                                            }                    <div class="metric-icon">"🚀"</div>

                                            Err(e) => {                    <div class="metric-content">

                                                set_error_msg.set(Some(format!("Retry failed: {}", e)));                        <h3>"Requests/sec"</h3>

                                            }                        <div class="metric-value">{move || format!("{}", metrics.get().requests_per_sec)}</div>

                                        }                        <div class="metric-change positive">"+12% from yesterday"</div>

                                        set_loading.set(false);                    </div>

                                    });                </div>

                                }>                

                                    "🔄 Try Again"                <div class="metric-card success">

                                </button>                    <div class="metric-icon">"✅"</div>

                            </div>                    <div class="metric-content">

                        </div>                        <h3>"Success Rate"</h3>

                    }.into_view()                        <div class="metric-value">{move || format!("{:.1}%", metrics.get().success_rate)}</div>

                }                        <div class="metric-change positive">"+0.2% from yesterday"</div>

                (false, None) => {                    </div>

                    let current_metrics = metrics.get();                </div>

                    view! {                

                        <div class="dashboard-grid">                <div class="metric-card warning">

                            {render_metrics_cards(current_metrics)}                    <div class="metric-icon">"⚡"</div>

                        </div>                    <div class="metric-content">

                    }.into_view()                        <h3>"Avg Latency"</h3>

                }                        <div class="metric-value">{move || format!("{:.1}ms", metrics.get().avg_latency)}</div>

            }}                        <div class="metric-change neutral">"-2ms from yesterday"</div>

        </div>                    </div>

    }                </div>

}                

                <div class="metric-card info">

fn render_metrics_cards(metrics: DashboardMetrics) -> impl IntoView {                    <div class="metric-icon">"🔗"</div>

    view! {                    <div class="metric-content">

        <div class="metrics-grid">                        <h3>"Active Routes"</h3>

            <div class="metric-card highlight">                        <div class="metric-value">{move || format!("{}", metrics.get().active_routes)}</div>

                <div class="metric-header">                        <div class="metric-change neutral">"No change"</div>

                    <span class="metric-icon">"⚡"</span>                    </div>

                    <span class="metric-title">"Requests/sec"</span>                </div>

                </div>            </div>

                <div class="metric-value">{metrics.requests_per_sec}</div>            

                <div class="metric-change positive">"↗ Live"</div>            <div class="dashboard-sections">

            </div>                <section class="system-status-section">

                                <h2>"🟢 System Status"</h2>

            <div class="metric-card">                    <div class="status-grid">

                <div class="metric-header">                        <div class="status-card healthy">

                    <span class="metric-icon">"✅"</span>                            <div class="status-header">

                    <span class="metric-title">"Success Rate"</span>                                <span class="status-label">"API Gateway"</span>

                </div>                                <span class="status-indicator">{move || health_status.get()}</span>

                <div class="metric-value">{format!("{:.1}%", metrics.success_rate)}</div>                            </div>

                <div class="metric-change positive">"↗ Good"</div>                            <div class="status-details">

            </div>                                <div class="status-metric">

                                                <span>"Uptime:"</span>

            <div class="metric-card">                                    <span>{move || metrics.get().uptime}</span>

                <div class="metric-header">                                </div>

                    <span class="metric-icon">"⏱️"</span>                                <div class="status-metric">

                    <span class="metric-title">"Avg Latency"</span>                                    <span>"Connections:"</span>

                </div>                                    <span>{move || format!("{}", metrics.get().active_connections)}</span>

                <div class="metric-value">{format!("{:.0}ms", metrics.avg_latency)}</div>                                </div>

                <div class="metric-change neutral">"→ Stable"</div>                            </div>

            </div>                        </div>

                                    

            <div class="metric-card">                        <div class="status-card healthy">

                <div class="metric-header">                            <div class="status-header">

                    <span class="metric-icon">"🛣️"</span>                                <span class="status-label">"Rate Limiter"</span>

                    <span class="metric-title">"Active Routes"</span>                                <span class="status-indicator">"Active"</span>

                </div>                            </div>

                <div class="metric-value">{metrics.active_routes}</div>                            <div class="status-details">

                <div class="metric-change neutral">"→ Ready"</div>                                <div class="status-metric">

            </div>                                    <span>"Limit:"</span>

                                                <span>"100 req/sec"</span>

            <div class="metric-card">                                </div>

                <div class="metric-header">                                <div class="status-metric">

                    <span class="metric-icon">"🔗"</span>                                    <span>"Burst:"</span>

                    <span class="metric-title">"Connections"</span>                                    <span>"200 requests"</span>

                </div>                                </div>

                <div class="metric-value">{metrics.active_connections}</div>                            </div>

                <div class="metric-change positive">"↗ Active"</div>                        </div>

            </div>                        

                                    <div class="status-card healthy">

            <div class="metric-card">                            <div class="status-header">

                <div class="metric-header">                                <span class="status-label">"Circuit Breaker"</span>

                    <span class="metric-icon">"⏰"</span>                                <span class="status-indicator">"Closed"</span>

                    <span class="metric-title">"Uptime"</span>                            </div>

                </div>                            <div class="status-details">

                <div class="metric-value uptime">{metrics.uptime}</div>                                <div class="status-metric">

                <div class="metric-change positive">"🟢 Healthy"</div>                                    <span>"Failure Threshold:"</span>

            </div>                                    <span>"5 failures"</span>

        </div>                                </div>

    }                                <div class="status-metric">

}                                    <span>"Timeout:"</span>

                                    <span>"30s"</span>

async fn fetch_dashboard_data() -> Result<(HealthStatus, MetricsSnapshot), ClientError> {                                </div>

    // Create client that works in WASM environment                            </div>

    let client = GatewayClient::new("http://localhost:5900")?;                        </div>

                        </div>

    // Try to get real data, fall back to mock data if gateway is not available                </section>

    match client.health().await {                

        Ok(health) => {                <section class="recent-activity-section">

            let metrics = client.metrics_snapshot().await?;                    <h2>"📊 Route Performance"</h2>

            Ok((health, metrics))                    <div class="route-list">

        }                        <div class="route-item">

        Err(_) => {                            <div class="route-path">"/api/identity/register/v3"</div>

            // Return mock data for development                            <div class="route-stats">

            let mock_health = HealthStatus {                                <span class="route-requests">"234 req/min"</span>

                status: "Mock Data".to_string(),                                <span class="route-latency">"15ms"</span>

                timestamp: chrono::Utc::now().to_rfc3339(),                                <span class="route-success success">"99.8%"</span>

                version: "0.2.6".to_string(),                            </div>

                uptime_seconds: 3661, // 1 hour, 1 minute, 1 second                        </div>

            };                        <div class="route-item">

            let mock_metrics = client.metrics_snapshot().await?;                            <div class="route-path">"/identity/register/v2"</div>

            Ok((mock_health, mock_metrics))                            <div class="route-stats">

        }                                <span class="route-requests">"89 req/min"</span>

    }                                <span class="route-latency">"45ms"</span>

}                                <span class="route-success success">"98.9%"</span>

                            </div>

fn calculate_rps(metrics: &MetricsSnapshot) -> u64 {                        </div>

    // Simple calculation - in production this would be more sophisticated                        <div class="route-item">

    metrics.requests_total / 60 // Assuming total_requests is per minute                            <div class="route-path">"/cats/{id}"</div>

}                            <div class="route-stats">

                                <span class="route-requests">"156 req/min"</span>

fn calculate_success_rate(metrics: &MetricsSnapshot) -> f64 {                                <span class="route-latency">"8ms"</span>

    if metrics.requests_total == 0 {                                <span class="route-success success">"100%"</span>

        100.0 // Default to 100% when no requests                            </div>

    } else {                        </div>

        (metrics.requests_success as f64 / metrics.requests_total as f64) * 100.0                        <div class="route-item">

    }                            <div class="route-path">"/protected/cats/{id}"</div>

}                            <div class="route-stats">

                                <span class="route-requests">"67 req/min"</span>

fn format_uptime(health: &HealthStatus) -> String {                                <span class="route-latency">"12ms"</span>

    let seconds = health.uptime_seconds;                                <span class="route-success warning">"95.2%"</span>

    match seconds {                            </div>

        0..=59 => format!("{}s", seconds),                        </div>

        60..=3599 => format!("{}m {}s", seconds / 60, seconds % 60),                    </div>

        3600..=86399 => format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60),                </section>

        _ => format!("{}d {}h", seconds / 86400, (seconds % 86400) / 3600),            </div>

    }        </div>

}    }
}

// Helper functions
async fn fetch_dashboard_data() -> Result<(HealthStatus, MetricsSnapshot), ClientError> {
    let client = GatewayClient::new("http://localhost:5900")?;
    
    let health = client.health().await?;
    let metrics = client.metrics_snapshot().await?;
    
    Ok((health, metrics))
}

fn calculate_rps(metrics: &MetricsSnapshot) -> u64 {
    // Simple calculation - in real app, would track over time window
    metrics.requests_total / 60 // Assume last minute
}

fn calculate_success_rate(metrics: &MetricsSnapshot) -> f64 {
    if metrics.requests_total == 0 {
        100.0
    } else {
        (metrics.requests_success as f64 / metrics.requests_total as f64) * 100.0
    }
}

fn format_uptime(health: &HealthStatus) -> String {
    let seconds = health.uptime_seconds;
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else {
        format!("{}d {}h", seconds / 86400, (seconds % 86400) / 3600)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn set_interval<F>(_f: F, _duration: std::time::Duration)
where
    F: Fn() + 'static,
{
    // No-op for server-side rendering
}

#[cfg(target_arch = "wasm32")]
fn set_interval<F>(f: F, duration: std::time::Duration)
where
    F: Fn() + 'static,
{
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    let window = web_sys::window().unwrap();
    let f = Closure::wrap(Box::new(f) as Box<dyn Fn()>);
    
    window
        .set_interval_with_callback_and_timeout_and_arguments_0(
            f.as_ref().unchecked_ref(),
            duration.as_millis() as i32,
        )
        .unwrap();
    
    f.forget(); // Keep the closure alive
}