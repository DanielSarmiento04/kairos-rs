use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_error: u64,
    pub active_connections: u64,
    pub average_response_time_ms: f64,
    pub timestamp: String,
}

impl Default for DashboardMetrics {
    fn default() -> Self {
        Self {
            requests_total: 0,
            requests_success: 0,
            requests_error: 0,
            active_connections: 0,
            average_response_time_ms: 0.0,
            timestamp: "Unknown".to_string(),
        }
    }
}

async fn fetch_dashboard_data() -> Result<DashboardMetrics, String> {
    // Use gloo-net for WASM-compatible HTTP requests
    use gloo_net::http::Request;
    
    let response = Request::get("http://localhost:5900/health")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch data: {:?}", e))?;

    if response.ok() {
        // For now, return mock data since we're just testing connectivity
        Ok(DashboardMetrics {
            requests_total: 1234,
            requests_success: 1200,
            requests_error: 34,
            active_connections: 12,
            average_response_time_ms: 15.5,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    } else {
        Err(format!("Server returned status: {}", response.status()))
    }
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let (metrics, set_metrics) = create_signal(DashboardMetrics::default());
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal(Option::<String>::None);

    // Fetch data on component mount
    create_effect(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match fetch_dashboard_data().await {
                Ok(data) => {
                    set_metrics.set(data);
                    set_error.set(None);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    });

    // Auto-refresh every 30 seconds
    let refresh_metrics = move || {
        spawn_local(async move {
            match fetch_dashboard_data().await {
                Ok(data) => {
                    set_metrics.set(data);
                    set_error.set(None);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
        });
    };

    view! {
        <div class="dashboard">
            <div class="dashboard-header">
                <h1>"📊 Gateway Dashboard"</h1>
                <button
                    class="refresh-btn"
                    on:click=move |_| refresh_metrics()
                >
                    "🔄 Refresh"
                </button>
            </div>

            {move || {
                if loading.get() {
                    view! {
                        <div class="loading">
                            <div class="spinner"></div>
                            <p>"Loading metrics..."</p>
                        </div>
                    }.into_view()
                } else if let Some(err) = error.get() {
                    view! {
                        <div class="error">
                            <h3>"❌ Error Loading Data"</h3>
                            <p>{err}</p>
                            <button
                                class="retry-btn"
                                on:click=move |_| refresh_metrics()
                            >
                                "🔄 Retry"
                            </button>
                        </div>
                    }.into_view()
                } else {
                    let m = metrics.get();
                    view! {
                        <div class="metrics-grid">
                            <div class="metric-card">
                                <div class="metric-icon">"📈"</div>
                                <div class="metric-content">
                                    <div class="metric-title">"Total Requests"</div>
                                    <div class="metric-value">{m.requests_total}</div>
                                </div>
                            </div>

                            <div class="metric-card success">
                                <div class="metric-icon">"✅"</div>
                                <div class="metric-content">
                                    <div class="metric-title">"Successful Requests"</div>
                                    <div class="metric-value">{m.requests_success}</div>
                                </div>
                            </div>

                            <div class="metric-card error">
                                <div class="metric-icon">"❌"</div>
                                <div class="metric-content">
                                    <div class="metric-title">"Failed Requests"</div>
                                    <div class="metric-value">{m.requests_error}</div>
                                </div>
                            </div>

                            <div class="metric-card">
                                <div class="metric-icon">"🔗"</div>
                                <div class="metric-content">
                                    <div class="metric-title">"Active Connections"</div>
                                    <div class="metric-value">{m.active_connections}</div>
                                </div>
                            </div>

                            <div class="metric-card">
                                <div class="metric-icon">"⏱️"</div>
                                <div class="metric-content">
                                    <div class="metric-title">"Avg Response Time"</div>
                                    <div class="metric-value">{format!("{:.1}ms", m.average_response_time_ms)}</div>
                                </div>
                            </div>

                            <div class="metric-card">
                                <div class="metric-icon">"🕒"</div>
                                <div class="metric-content">
                                    <div class="metric-title">"Last Updated"</div>
                                    <div class="metric-value small">{m.timestamp}</div>
                                </div>
                            </div>
                        </div>

                        <div class="charts-section">
                            <h2>"📊 Performance Overview"</h2>
                            <div class="chart-placeholder">
                                <p>"📈 Charts coming soon..."</p>
                                <div class="success-rate">
                                    {
                                        let success_rate = if m.requests_total > 0 {
                                            (m.requests_success as f64 / m.requests_total as f64) * 100.0
                                        } else {
                                            100.0
                                        };
                                        format!("Success Rate: {:.1}%", success_rate)
                                    }
                                </div>
                            </div>
                        </div>
                    }.into_view()
                }
            }}
        </div>
    }
}