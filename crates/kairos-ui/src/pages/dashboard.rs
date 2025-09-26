use leptos::*;
use leptos::logging::log;
use kairos_client::{GatewayClient, ClientError, MetricsSnapshot, HealthStatus};
use wasm_bindgen_futures::spawn_local;

#[derive(Debug, Clone)]
pub struct DashboardMetrics {
    pub requests_per_sec: u64,
    pub success_rate: f64,
    pub avg_latency: f64,
    pub active_routes: u64,
    pub active_connections: u64,
    pub uptime: String,
}

impl Default for DashboardMetrics {
    fn default() -> Self {
        Self {
            requests_per_sec: 0,
            success_rate: 0.0,
            avg_latency: 0.0,
            active_routes: 0,
            active_connections: 0,
            uptime: "Loading...".to_string(),
        }
    }
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let (metrics, set_metrics) = create_signal(DashboardMetrics::default());
    let (health_status, set_health_status) = create_signal("Loading...".to_string());
    let (loading, set_loading) = create_signal(true);
    let (error_msg, set_error_msg) = create_signal(None::<String>);

    // Fetch data on component mount
    create_effect(move |_| {
        spawn_local(async move {
            match fetch_dashboard_data().await {
                Ok((health, metrics_data)) => {
                    let health_status = health.status.clone();
                    let uptime = format_uptime(&health);
                    
                    set_health_status.set(health_status);
                    set_metrics.set(DashboardMetrics {
                        requests_per_sec: calculate_rps(&metrics_data),
                        success_rate: calculate_success_rate(&metrics_data),
                        avg_latency: metrics_data.average_response_time_ms,
                        active_routes: 4, // From config - TODO: make dynamic
                        active_connections: metrics_data.active_connections,
                        uptime,
                    });
                    set_error_msg.set(None);
                }
                Err(e) => {
                    log!("Error fetching dashboard data: {:?}", e);
                    set_error_msg.set(Some(format!("Failed to connect to gateway: {}", e)));
                }
            }
            set_loading.set(false);
        });
    });

    // Auto-refresh every 30 seconds
    let _refresh_interval = create_effect(move |_| {
        set_interval(
            move || {
                spawn_local(async move {
                    if let Ok((health, metrics_data)) = fetch_dashboard_data().await {
                        let health_status = health.status.clone();
                        let uptime = format_uptime(&health);
                        
                        set_health_status.set(health_status);
                        set_metrics.set(DashboardMetrics {
                            requests_per_sec: calculate_rps(&metrics_data),
                            success_rate: calculate_success_rate(&metrics_data),
                            avg_latency: metrics_data.average_response_time_ms,
                            active_routes: 4,
                            active_connections: metrics_data.active_connections,
                            uptime,
                        });
                    }
                });
            },
            std::time::Duration::from_secs(30),
        );
    });

    view! {
        <div class="dashboard">
            <div class="page-header">
                <h1>"🚀 Dashboard"</h1>
                <p>"Real-time overview of your Kairos API Gateway"</p>
                <div class="header-actions">
                    <button 
                        class="refresh-btn"
                        on:click=move |_| {
                            set_loading.set(true);
                            spawn_local(async move {
                                if let Ok((health, metrics_data)) = fetch_dashboard_data().await {
                                    let health_status = health.status.clone();
                                    let uptime = format_uptime(&health);
                                    
                                    set_health_status.set(health_status);
                                    set_metrics.set(DashboardMetrics {
                                        requests_per_sec: calculate_rps(&metrics_data),
                                        success_rate: calculate_success_rate(&metrics_data),
                                        avg_latency: metrics_data.average_response_time_ms,
                                        active_routes: 4,
                                        active_connections: metrics_data.active_connections,
                                        uptime,
                                    });
                                    set_error_msg.set(None);
                                }
                                set_loading.set(false);
                            });
                        }
                    >
                        {move || if loading.get() { "🔄 Refreshing..." } else { "🔄 Refresh" }}
                    </button>
                </div>
            </div>
            
            {move || error_msg.get().map(|err| view! {
                <div class="error-banner">
                    <span class="error-icon">"⚠️"</span>
                    <span>{err}</span>
                    <span class="error-hint">"Make sure the gateway is running on port 5900"</span>
                </div>
            })}
            
            <div class="dashboard-grid">
                <div class="metric-card primary">
                    <div class="metric-icon">"🚀"</div>
                    <div class="metric-content">
                        <h3>"Requests/sec"</h3>
                        <div class="metric-value">{move || format!("{}", metrics.get().requests_per_sec)}</div>
                        <div class="metric-change positive">"+12% from yesterday"</div>
                    </div>
                </div>
                
                <div class="metric-card success">
                    <div class="metric-icon">"✅"</div>
                    <div class="metric-content">
                        <h3>"Success Rate"</h3>
                        <div class="metric-value">{move || format!("{:.1}%", metrics.get().success_rate)}</div>
                        <div class="metric-change positive">"+0.2% from yesterday"</div>
                    </div>
                </div>
                
                <div class="metric-card warning">
                    <div class="metric-icon">"⚡"</div>
                    <div class="metric-content">
                        <h3>"Avg Latency"</h3>
                        <div class="metric-value">{move || format!("{:.1}ms", metrics.get().avg_latency)}</div>
                        <div class="metric-change neutral">"-2ms from yesterday"</div>
                    </div>
                </div>
                
                <div class="metric-card info">
                    <div class="metric-icon">"🔗"</div>
                    <div class="metric-content">
                        <h3>"Active Routes"</h3>
                        <div class="metric-value">{move || format!("{}", metrics.get().active_routes)}</div>
                        <div class="metric-change neutral">"No change"</div>
                    </div>
                </div>
            </div>
            
            <div class="dashboard-sections">
                <section class="system-status-section">
                    <h2>"🟢 System Status"</h2>
                    <div class="status-grid">
                        <div class="status-card healthy">
                            <div class="status-header">
                                <span class="status-label">"API Gateway"</span>
                                <span class="status-indicator">{move || health_status.get()}</span>
                            </div>
                            <div class="status-details">
                                <div class="status-metric">
                                    <span>"Uptime:"</span>
                                    <span>{move || metrics.get().uptime}</span>
                                </div>
                                <div class="status-metric">
                                    <span>"Connections:"</span>
                                    <span>{move || format!("{}", metrics.get().active_connections)}</span>
                                </div>
                            </div>
                        </div>
                        
                        <div class="status-card healthy">
                            <div class="status-header">
                                <span class="status-label">"Rate Limiter"</span>
                                <span class="status-indicator">"Active"</span>
                            </div>
                            <div class="status-details">
                                <div class="status-metric">
                                    <span>"Limit:"</span>
                                    <span>"100 req/sec"</span>
                                </div>
                                <div class="status-metric">
                                    <span>"Burst:"</span>
                                    <span>"200 requests"</span>
                                </div>
                            </div>
                        </div>
                        
                        <div class="status-card healthy">
                            <div class="status-header">
                                <span class="status-label">"Circuit Breaker"</span>
                                <span class="status-indicator">"Closed"</span>
                            </div>
                            <div class="status-details">
                                <div class="status-metric">
                                    <span>"Failure Threshold:"</span>
                                    <span>"5 failures"</span>
                                </div>
                                <div class="status-metric">
                                    <span>"Timeout:"</span>
                                    <span>"30s"</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>
                
                <section class="recent-activity-section">
                    <h2>"📊 Route Performance"</h2>
                    <div class="route-list">
                        <div class="route-item">
                            <div class="route-path">"/api/identity/register/v3"</div>
                            <div class="route-stats">
                                <span class="route-requests">"234 req/min"</span>
                                <span class="route-latency">"15ms"</span>
                                <span class="route-success success">"99.8%"</span>
                            </div>
                        </div>
                        <div class="route-item">
                            <div class="route-path">"/identity/register/v2"</div>
                            <div class="route-stats">
                                <span class="route-requests">"89 req/min"</span>
                                <span class="route-latency">"45ms"</span>
                                <span class="route-success success">"98.9%"</span>
                            </div>
                        </div>
                        <div class="route-item">
                            <div class="route-path">"/cats/{id}"</div>
                            <div class="route-stats">
                                <span class="route-requests">"156 req/min"</span>
                                <span class="route-latency">"8ms"</span>
                                <span class="route-success success">"100%"</span>
                            </div>
                        </div>
                        <div class="route-item">
                            <div class="route-path">"/protected/cats/{id}"</div>
                            <div class="route-stats">
                                <span class="route-requests">"67 req/min"</span>
                                <span class="route-latency">"12ms"</span>
                                <span class="route-success warning">"95.2%"</span>
                            </div>
                        </div>
                    </div>
                </section>
            </div>
        </div>
    }
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