use leptos::*;
use leptos::logging::log;
use kairos_client::{GatewayClient, ClientError, MetricsSnapshot, HealthStatus};
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;

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
    let (health_status, set_health_status) = create_signal(None::<String>);
    let (last_updated, set_last_updated) = create_signal("Never".to_string());
    let (is_loading, set_is_loading) = create_signal(true);
    let (error_message, set_error_message) = create_signal(None::<String>);

    // Auto-refresh interval (5 seconds)
    let refresh_interval = std::time::Duration::from_secs(5);

    // Function to fetch data from Kairos Gateway
    let fetch_data = move || {
        set_is_loading.set(true);
        set_error_message.set(None);
        
        spawn_local(async move {
            match GatewayClient::new("http://localhost:8080") {
                Ok(client) => {
                    // Fetch health status
                    match client.health().await {
                        Ok(health) => {
                            set_health_status.set(Some(health.status));
                        }
                        Err(e) => {
                            log!("Failed to fetch health: {:?}", e);
                            set_error_message.set(Some(format!("Health check failed: {}", e)));
                        }
                    }

                    // Fetch metrics
                    match client.metrics_snapshot().await {
                        Ok(snapshot) => {
                            let dashboard_metrics = DashboardMetrics {
                                requests_per_sec: snapshot.requests_total / 60, // Rough estimate
                                success_rate: if snapshot.requests_total > 0 {
                                    (snapshot.requests_success as f64 / snapshot.requests_total as f64) * 100.0
                                } else {
                                    100.0
                                },
                                avg_latency: snapshot.average_response_time_ms,
                                active_routes: 5, // Mock data for now
                                active_connections: snapshot.active_connections,
                                uptime: chrono::Utc::now().format("%H:%M:%S").to_string(),
                            };
                            
                            set_metrics.set(dashboard_metrics);
                            set_last_updated.set(chrono::Utc::now().format("%H:%M:%S").to_string());
                            set_error_message.set(None);
                        }
                        Err(e) => {
                            log!("Failed to fetch metrics: {:?}", e);
                            set_error_message.set(Some(format!("Metrics fetch failed: {}", e)));
                            
                            // Fall back to mock data
                            let mock_metrics = DashboardMetrics {
                                requests_per_sec: 150,
                                success_rate: 99.5,
                                avg_latency: 45.0,
                                active_routes: 5,
                                active_connections: 25,
                                uptime: "2h 15m".to_string(),
                            };
                            set_metrics.set(mock_metrics);
                        }
                    }
                }
                Err(e) => {
                    log!("Failed to create client: {:?}", e);
                    set_error_message.set(Some(format!("Client creation failed: {}", e)));
                    
                    // Fall back to mock data
                    let mock_metrics = DashboardMetrics {
                        requests_per_sec: 150,
                        success_rate: 99.5,
                        avg_latency: 45.0,
                        active_routes: 5,
                        active_connections: 25,
                        uptime: "2h 15m".to_string(),
                    };
                    set_metrics.set(mock_metrics);
                }
            }
            
            set_is_loading.set(false);
        });
    };

    // Initial data fetch
    create_effect(move |_| {
        fetch_data();
    });

    // For now, skip auto-refresh to simplify - user can manually refresh

    view! {
        <div class="dashboard">
            <div class="dashboard-header">
                <h1>"Kairos Gateway Dashboard"</h1>
                <div class="refresh-info">
                    <span class="last-updated">"Last updated: " {move || last_updated.get()}</span>
                    <button 
                        class="refresh-btn"
                        on:click=move |_| fetch_data()
                        disabled=move || is_loading.get()
                    >
                        {move || if is_loading.get() { "Refreshing..." } else { "Refresh" }}
                    </button>
                </div>
            </div>

            // Error message display
            {move || error_message.get().map(|msg| view! {
                <div class="error-banner">
                    <span class="error-icon">"⚠️"</span>
                    <span class="error-text">{msg}</span>
                </div>
            })}

            // Health Status
            <div class="health-status">
                <h2>"System Health"</h2>
                <div class="status-indicator">
                    {move || match health_status.get() {
                        Some(status) => view! {
                            <span class=format!("status-badge {}", 
                                if status == "healthy" { "healthy" } else { "unhealthy" }
                            )>
                                {status}
                            </span>
                        },
                        None => view! {
                            <span class="status-badge unknown">"Unknown"</span>
                        }
                    }}
                </div>
            </div>

            // Metrics Grid
            <div class="metrics-grid">
                <div class="metric-card">
                    <div class="metric-header">
                        <h3>"Requests/sec"</h3>
                        <span class="metric-icon">"📊"</span>
                    </div>
                    <div class="metric-value">
                        {move || metrics.get().requests_per_sec}
                    </div>
                    <div class="metric-trend positive">"↗ +12%"</div>
                </div>

                <div class="metric-card">
                    <div class="metric-header">
                        <h3>"Success Rate"</h3>
                        <span class="metric-icon">"✅"</span>
                    </div>
                    <div class="metric-value">
                        {move || format!("{:.1}%", metrics.get().success_rate)}
                    </div>
                    <div class="metric-trend positive">"↗ +0.2%"</div>
                </div>

                <div class="metric-card">
                    <div class="metric-header">
                        <h3>"Avg Latency"</h3>
                        <span class="metric-icon">"⚡"</span>
                    </div>
                    <div class="metric-value">
                        {move || format!("{:.0}ms", metrics.get().avg_latency)}
                    </div>
                    <div class="metric-trend neutral">"→ 0%"</div>
                </div>

                <div class="metric-card">
                    <div class="metric-header">
                        <h3>"Active Routes"</h3>
                        <span class="metric-icon">"🛣️"</span>
                    </div>
                    <div class="metric-value">
                        {move || metrics.get().active_routes}
                    </div>
                    <div class="metric-trend neutral">"→ 0%"</div>
                </div>

                <div class="metric-card">
                    <div class="metric-header">
                        <h3>"Connections"</h3>
                        <span class="metric-icon">"🔗"</span>
                    </div>
                    <div class="metric-value">
                        {move || metrics.get().active_connections}
                    </div>
                    <div class="metric-trend negative">"↘ -3%"</div>
                </div>

                <div class="metric-card">
                    <div class="metric-header">
                        <h3>"Uptime"</h3>
                        <span class="metric-icon">"⏰"</span>
                    </div>
                    <div class="metric-value uptime">
                        {move || metrics.get().uptime}
                    </div>
                    <div class="metric-trend positive">"Running"</div>
                </div>
            </div>

            // Recent Activity
            <div class="recent-activity">
                <h2>"Recent Activity"</h2>
                <div class="activity-list">
                    <div class="activity-item">
                        <span class="activity-time">"12:34"</span>
                        <span class="activity-message">"New route registered: /api/users"</span>
                        <span class="activity-status success">"✓"</span>
                    </div>
                    <div class="activity-item">
                        <span class="activity-time">"12:31"</span>
                        <span class="activity-message">"Rate limit exceeded for IP 192.168.1.100"</span>
                        <span class="activity-status warning">"⚠"</span>
                    </div>
                    <div class="activity-item">
                        <span class="activity-time">"12:28"</span>
                        <span class="activity-message">"Circuit breaker opened for /api/external"</span>
                        <span class="activity-status error">"✗"</span>
                    </div>
                    <div class="activity-item">
                        <span class="activity-time">"12:25"</span>
                        <span class="activity-message">"Configuration reloaded successfully"</span>
                        <span class="activity-status success">"✓"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}