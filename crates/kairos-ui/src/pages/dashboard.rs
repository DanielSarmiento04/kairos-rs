//! Dashboard page with real-time metrics and system status.

use crate::components::{StatusVariant, *};
use crate::models::*;
use crate::server_functions::*;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;
#[cfg(feature = "hydrate")]
use web_sys::{MessageEvent, WebSocket};

/// Main dashboard page displaying gateway health and metrics.
#[component]
pub fn DashboardPage() -> impl IntoView {
    // Auto-refresh signal (triggers every 30 seconds)
    let refresh_trigger = RwSignal::new(0u32);
    // Live WebSocket Metrics
    let (live_metrics, set_live_metrics) = signal::<Option<MetricsData>>(None);
    let (ws_status, set_ws_status) = signal("🟠 Polling API");
    
    // Set up auto-refresh timer & WebSocket hooks
    Effect::new(move |_| {
        spawn_local(async move {
            #[cfg(feature = "hydrate")]
            {
                // Connect to real-time Metrics Streaming API
                if let Ok(host) = web_sys::window().unwrap().location().hostname() {
                    let port = 5900; // Future enhancement: fetch from config
                    let ws_url = format!("ws://{}:{}/ws/metrics", host, port);
                    
                    if let Ok(ws) = WebSocket::new(&ws_url) {
                        let onmessage = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                                let text: String = txt.into();
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if json["type"] == "Snapshot" {
                                        set_live_metrics.update(|m| {
                                            let mut mm = m.clone().unwrap_or_default();
                                            if let Some(n) = json["requests_total"].as_u64() { mm.requests_total = n; }
                                            if let Some(n) = json["avg_response_time"].as_f64() { mm.response_time_avg = n; }
                                            if let Some(n) = json["success_rate"].as_f64() { mm.success_rate = n; }
                                            if let Some(n) = json["active_connections"].as_u64() { 
                                                mm.active_connections = n as u32; 
                                                if mm.active_connections > mm.peak_connections { mm.peak_connections = mm.active_connections; }
                                            }
                                            if let Some(n) = json["requests_error"].as_u64() { 
                                                if n > mm.http_5xx_errors { mm.http_5xx_errors = n; }
                                            }
                                            *m = Some(mm);
                                        });
                                    }
                                }
                            }
                        });
                        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                        onmessage.forget();
                        
                        let onopen = Closure::<dyn FnMut(_)>::new(move |_| {
                            set_ws_status.set("🟢 Live Streaming");
                            // Auto-fetch the heavier payloads once WEBSOCKET is active
                            refresh_trigger.update(|n| *n += 1);
                        });
                        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                        onopen.forget();
                        
                        let onclose = Closure::<dyn FnMut(_)>::new(move |_| {
                            set_ws_status.set("🔴 Disconnected");
                        });
                        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
                        onclose.forget();
                    }
                }

                // Polling Loop Fallback for detailed analytics
                loop {
                    gloo_timers::future::TimeoutFuture::new(30_000).await;
                    refresh_trigger.update(|n| *n += 1);
                }
            }
            #[cfg(not(feature = "hydrate"))]
            {
                // On server, just break
            }
        });
    });

    // Fetch health data
    let health_resource = Resource::new(
        move || refresh_trigger.get(),
        |_| async move { get_health().await },
    );

    // Fetch metrics data
    let metrics_resource = Resource::new(
        move || refresh_trigger.get(),
        |_| async move { get_metrics().await },
    );

    view! {
        <div class="dashboard">
            <div class="dashboard-header">
                <h1 class="page-title">"Gateway Dashboard"</h1>
                <p class="page-subtitle">"Real-time monitoring and system status"</p>

                <button
                    class="btn btn-secondary refresh-btn"
                    on:click=move |_| refresh_trigger.update(|n| *n += 1)
                >
                    "🔄 Refresh"
                </button>
            </div>

            // Health Status Section
            <section class="dashboard-section">
                <h2 class="section-title">"System Health"</h2>

                <Suspense fallback=move || view! { <LoadingSpinner message="Loading health status...".to_string() /> }>
                    {move || {
                        health_resource.get().map(|result| match result {
                            Ok(health) => view! {
                                <div class="health-overview">
                                    <MetricCard
                                        title="Status".to_string()
                                        value=health.status.clone()
                                        icon=if health.is_healthy() { "✅".to_string() } else { "❌".to_string() }
                                    />

                                    <MetricCard
                                        title="Version".to_string()
                                        value=health.version.clone()
                                        icon="🏷️".to_string()
                                    />

                                    <MetricCard
                                        title="Uptime".to_string()
                                        value=health.format_uptime()
                                        icon="⏱️".to_string()
                                    />
                                </div>
                            }.into_any(),
                            Err(e) => view! {
                                <ErrorBoundaryView
                                    error=format!("{}", e)
                                    title="Failed to load health status".to_string()
                                />
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </section>

            // Request Metrics Section
            <section class="dashboard-section">
                <h2 class="section-title">"Request Metrics"</h2>

                <Suspense fallback=move || view! { <LoadingSpinner message="Loading metrics...".to_string() /> }>
                    {move || {
                        metrics_resource.get().map(|result| match result {
                            Ok(polled_metrics) => {
                                let metrics = if let Some(live) = live_metrics.get() {
                                    let mut m = polled_metrics.clone();
                                    if live.requests_total > m.requests_total { m.requests_total = live.requests_total; }
                                    m.active_connections = live.active_connections;
                                    if m.active_connections > m.peak_connections { m.peak_connections = m.active_connections; }
                                    m.success_rate = live.success_rate;
                                    m.response_time_avg = live.response_time_avg;
                                    if live.http_5xx_errors > m.http_5xx_errors { m.http_5xx_errors = live.http_5xx_errors; }
                                    m
                                } else {
                                    polled_metrics.clone()
                                };
                                let success_trend = if metrics.success_rate >= 95.0 { "up".to_string() } else if metrics.success_rate >= 80.0 { "neutral".to_string() } else { "down".to_string() };
                                let response_trend = if metrics.response_time_avg < 100.0 { "up".to_string() } else if metrics.response_time_avg < 500.0 { "neutral".to_string() } else { "down".to_string() };
                                let peak_subtitle = format!("Peak: {}", metrics.peak_connections);

                                view! {
                                <div class="metrics-grid">
                                    <MetricCard
                                        title="Total Requests".to_string()
                                        value=format!("{}", metrics.requests_total)
                                        icon="📊".to_string()
                                    />

                                    <MetricCard
                                        title="Success Rate".to_string()
                                        value=format!("{:.2}%", metrics.success_rate)
                                        icon="✅".to_string()
                                        trend=success_trend
                                    />

                                    <MetricCard
                                        title="Avg Response Time".to_string()
                                        value=format!("{:.2}ms", metrics.response_time_avg)
                                        icon="⚡".to_string()
                                        trend=response_trend
                                    />

                                    <MetricCard
                                        title="Active Connections".to_string()
                                        value=format!("{}", metrics.active_connections)
                                        icon="🔗".to_string()
                                        subtitle=peak_subtitle
                                    />
                                </div>

                                // Error Breakdown
                                <div class="error-breakdown">
                                    <h3 class="subsection-title">"Error Breakdown"</h3>
                                    <div class="metrics-grid">
                                        <MetricCard
                                            title="4xx Errors".to_string()
                                            value=format!("{}", metrics.http_4xx_errors)
                                            icon="⚠️".to_string()
                                        />

                                        <MetricCard
                                            title="5xx Errors".to_string()
                                            value=format!("{}", metrics.http_5xx_errors)
                                            icon="❌".to_string()
                                        />

                                        <MetricCard
                                            title="Timeouts".to_string()
                                            value=format!("{}", metrics.timeout_errors)
                                            icon="⏱️".to_string()
                                        />

                                        <MetricCard
                                            title="Connection Errors".to_string()
                                            value=format!("{}", metrics.connection_errors)
                                            icon="🔌".to_string()
                                        />
                                    </div>
                                </div>

                                // Response Time Distribution
                                <div class="response-time-distribution">
                                    <h3 class="subsection-title">"Response Time Distribution"</h3>
                                    <div class="histogram">
                                        <div class="histogram-bar">
                                            <span class="histogram-label">"< 100ms"</span>
                                            <div class="histogram-bar-container">
                                                <div
                                                    class="histogram-bar-fill histogram-bar-excellent"
                                                    style=format!("width: {}%", calculate_percentage(metrics.response_time_bucket_100ms, metrics.requests_total))
                                                ></div>
                                            </div>
                                            <span class="histogram-value">{metrics.response_time_bucket_100ms}</span>
                                        </div>

                                        <div class="histogram-bar">
                                            <span class="histogram-label">"< 500ms"</span>
                                            <div class="histogram-bar-container">
                                                <div
                                                    class="histogram-bar-fill histogram-bar-good"
                                                    style=format!("width: {}%", calculate_percentage(metrics.response_time_bucket_500ms - metrics.response_time_bucket_100ms, metrics.requests_total))
                                                ></div>
                                            </div>
                                            <span class="histogram-value">{metrics.response_time_bucket_500ms - metrics.response_time_bucket_100ms}</span>
                                        </div>

                                        <div class="histogram-bar">
                                            <span class="histogram-label">"< 1s"</span>
                                            <div class="histogram-bar-container">
                                                <div
                                                    class="histogram-bar-fill histogram-bar-fair"
                                                    style=format!("width: {}%", calculate_percentage(metrics.response_time_bucket_1s - metrics.response_time_bucket_500ms, metrics.requests_total))
                                                ></div>
                                            </div>
                                            <span class="histogram-value">{metrics.response_time_bucket_1s - metrics.response_time_bucket_500ms}</span>
                                        </div>

                                        <div class="histogram-bar">
                                            <span class="histogram-label">"< 5s"</span>
                                            <div class="histogram-bar-container">
                                                <div
                                                    class="histogram-bar-fill histogram-bar-poor"
                                                    style=format!("width: {}%", calculate_percentage(metrics.response_time_bucket_5s - metrics.response_time_bucket_1s, metrics.requests_total))
                                                ></div>
                                            </div>
                                            <span class="histogram-value">{metrics.response_time_bucket_5s - metrics.response_time_bucket_1s}</span>
                                        </div>

                                        <div class="histogram-bar">
                                            <span class="histogram-label">"> 5s"</span>
                                            <div class="histogram-bar-container">
                                                <div
                                                    class="histogram-bar-fill histogram-bar-critical"
                                                    style=format!("width: {}%", calculate_percentage(metrics.response_time_bucket_inf, metrics.requests_total))
                                                ></div>
                                            </div>
                                            <span class="histogram-value">{metrics.response_time_bucket_inf}</span>
                                        </div>
                                    </div>
                                </div>

                                // Circuit Breakers
                                {if !metrics.circuit_breakers.is_empty() {
                                    view! {
                                        <div class="circuit-breakers">
                                            <h3 class="subsection-title">"Circuit Breakers"</h3>
                                            <div class="circuit-breaker-list">
                                                {metrics.circuit_breakers.iter().map(|cb| {
                                                    let variant = if cb.state.is_healthy() {
                                                        StatusVariant::Success
                                                    } else {
                                                        StatusVariant::Error
                                                    };

                                                    view! {
                                                        <div class="circuit-breaker-item">
                                                            <span class="cb-service">{cb.route.clone()}</span>
                                                            <StatusBadge
                                                                text=format!("{:?}", cb.state)
                                                                variant=variant
                                                            />
                                                            <span class="cb-stats">
                                                                {format!("✅ {} | ❌ {}", cb.success_count, cb.failure_count)}
                                                            </span>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
                                }}

                                // Data Transfer
                                <div class="data-transfer">
                                    <h3 class="subsection-title">"Data Transfer"</h3>
                                    <div class="metrics-grid">
                                        <MetricCard
                                            title="Request Data".to_string()
                                            value=MetricsData::format_bytes(metrics.request_bytes_total)
                                            icon="⬇️".to_string()
                                        />

                                        <MetricCard
                                            title="Response Data".to_string()
                                            value=MetricsData::format_bytes(metrics.response_bytes_total)
                                            icon="⬆️".to_string()
                                        />
                                    </div>
                                </div>
                            }.into_any()
                            },
                            Err(e) => view! {
                                <ErrorBoundaryView
                                    error=format!("{}", e)
                                    title="Failed to load metrics".to_string()
                                />
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </section>

            <div class="dashboard-footer">
                <p class="auto-refresh-notice">"Auto-refreshes every 30 seconds"</p>
            </div>
        </div>
    }
}

/// Helper function to calculate percentage for histogram bars
fn calculate_percentage(value: u64, total: u64) -> u64 {
    if total == 0 {
        0
    } else {
        (value * 100) / total
    }
}
