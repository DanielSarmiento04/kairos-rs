//! Advanced metrics visualization page with detailed analytics.

use crate::components::*;
use crate::models::*;
use crate::server_functions::*;
use chrono::{Duration, Utc};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[derive(Clone, Copy, PartialEq)]
enum MetricsView {
    Overview,
    RealTime,
    Performance,
    Errors,
    Traffic,
    CircuitBreakers,
    Historical,
}

/// Advanced metrics page with detailed charts and analytics.
#[component]
pub fn MetricsPage() -> impl IntoView {
    let (active_view, set_active_view) = signal(MetricsView::Overview);
    let refresh_trigger = RwSignal::new(0u32);

    // Auto-refresh every 10 seconds
    spawn_local(async move {
        loop {
            #[cfg(feature = "hydrate")]
            {
                gloo_timers::future::TimeoutFuture::new(10_000).await;
                refresh_trigger.update(|n| *n += 1);
            }
            #[cfg(not(feature = "hydrate"))]
            {
                break;
            }
        }
    });

    // Fetch metrics data
    let metrics_resource = Resource::new(
        move || refresh_trigger.get(),
        |_| async move { get_metrics().await },
    );

    view! {
        <div class="metrics-page">
            <div class="page-header">
                <h1 class="page-title">"📊 Metrics & Analytics"</h1>
                <p class="page-subtitle">"Detailed performance metrics and analytics"</p>

                <div class="header-actions">
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| refresh_trigger.update(|n| *n += 1)
                    >
                        "🔄 Refresh"
                    </button>
                    <span class="auto-refresh-notice">"Auto-refreshes every 10s"</span>
                </div>
            </div>

            // Metrics View Tabs
            <div class="metrics-tabs">
                <button
                    class=move || if active_view.get() == MetricsView::Overview { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_view.set(MetricsView::Overview)
                >
                    "📈 Overview"
                </button>
                <button
                    class=move || if active_view.get() == MetricsView::RealTime { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_view.set(MetricsView::RealTime)
                >
                    "⏱️ Real-Time"
                </button>
                <button
                    class=move || if active_view.get() == MetricsView::Performance { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_view.set(MetricsView::Performance)
                >
                    "⚡ Performance"
                </button>
                <button
                    class=move || if active_view.get() == MetricsView::Errors { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_view.set(MetricsView::Errors)
                >
                    "⚠️ Errors"
                </button>
                <button
                    class=move || if active_view.get() == MetricsView::Traffic { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_view.set(MetricsView::Traffic)
                >
                    "🌐 Traffic"
                </button>
                <button
                    class=move || if active_view.get() == MetricsView::CircuitBreakers { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_view.set(MetricsView::CircuitBreakers)
                >
                    "🔌 Circuit Breakers"
                </button>
                <button
                    class=move || if active_view.get() == MetricsView::Historical { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_view.set(MetricsView::Historical)
                >
                    "📅 Historical"
                </button>
            </div>

            // Metrics Content
            <div class="metrics-content">
                <Suspense fallback=move || view! { <LoadingSpinner message="Loading metrics data...".to_string() /> }>
                    {move || {
                        metrics_resource.get().map(|result| match result {
                            Ok(metrics) => {
                                match active_view.get() {
                                    MetricsView::Overview => view! { <OverviewView metrics=metrics /> }.into_any(),
                                    MetricsView::RealTime => view! { <RealTimeMetrics /> }.into_any(),
                                    MetricsView::Performance => view! { <PerformanceView metrics=metrics /> }.into_any(),
                                    MetricsView::Errors => view! { <ErrorsView metrics=metrics /> }.into_any(),
                                    MetricsView::Traffic => view! { <TrafficView metrics=metrics /> }.into_any(),
                                    MetricsView::CircuitBreakers => view! { <CircuitBreakersView metrics=metrics /> }.into_any(),
                                    MetricsView::Historical => view! { <HistoricalView /> }.into_any(),
                                }
                            }
                            Err(e) => view! {
                                <ErrorBoundaryView
                                    error=format!("{}", e)
                                    title="Failed to load metrics".to_string()
                                />
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}

// ============================================================================
// Overview View
// ============================================================================

#[component]
fn OverviewView(metrics: MetricsData) -> impl IntoView {
    let total_errors = metrics.http_4xx_errors
        + metrics.http_5xx_errors
        + metrics.timeout_errors
        + metrics.connection_errors;
    let error_rate = if metrics.requests_total > 0 {
        (total_errors as f64 / metrics.requests_total as f64) * 100.0
    } else {
        0.0
    };

    view! {
        <div class="overview-view">
            <div class="metrics-summary">
                <div class="summary-card primary-card">
                    <div class="summary-icon">"📊"</div>
                    <div class="summary-content">
                        <div class="summary-value">{format!("{}", metrics.requests_total)}</div>
                        <div class="summary-label">"Total Requests"</div>
                    </div>
                </div>

                <div class="summary-card success-card">
                    <div class="summary-icon">"✅"</div>
                    <div class="summary-content">
                        <div class="summary-value">{format!("{:.2}%", metrics.success_rate)}</div>
                        <div class="summary-label">"Success Rate"</div>
                    </div>
                </div>

                <div class="summary-card warning-card">
                    <div class="summary-icon">"⚡"</div>
                    <div class="summary-content">
                        <div class="summary-value">{format!("{:.0}ms", metrics.response_time_avg)}</div>
                        <div class="summary-label">"Avg Response Time"</div>
                    </div>
                </div>

                <div class="summary-card error-card">
                    <div class="summary-icon">"❌"</div>
                    <div class="summary-content">
                        <div class="summary-value">{format!("{:.2}%", error_rate)}</div>
                        <div class="summary-label">"Error Rate"</div>
                    </div>
                </div>
            </div>

            <div class="metrics-grid">
                <div class="metric-panel">
                    <h3>"Connection Statistics"</h3>
                    <div class="panel-content">
                        <div class="stat-row">
                            <span class="stat-label">"Active Connections:"</span>
                            <span class="stat-value">{metrics.active_connections}</span>
                        </div>
                        <div class="stat-row">
                            <span class="stat-label">"Peak Connections:"</span>
                            <span class="stat-value">{metrics.peak_connections}</span>
                        </div>
                        <div class="stat-row">
                            <span class="stat-label">"Requests In Flight:"</span>
                            <span class="stat-value">{metrics.requests_in_flight}</span>
                        </div>
                    </div>
                </div>

                <div class="metric-panel">
                    <h3>"Data Transfer"</h3>
                    <div class="panel-content">
                        <div class="stat-row">
                            <span class="stat-label">"Request Data:"</span>
                            <span class="stat-value">{MetricsData::format_bytes(metrics.request_bytes_total)}</span>
                        </div>
                        <div class="stat-row">
                            <span class="stat-label">"Response Data:"</span>
                            <span class="stat-value">{MetricsData::format_bytes(metrics.response_bytes_total)}</span>
                        </div>
                        <div class="stat-row">
                            <span class="stat-label">"Total Transferred:"</span>
                            <span class="stat-value">{MetricsData::format_bytes(metrics.data_transferred_bytes)}</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// Performance View
// ============================================================================

#[component]
fn PerformanceView(metrics: MetricsData) -> impl IntoView {
    let total = metrics.requests_total.max(1);

    view! {
        <div class="performance-view">
            <h2>"Response Time Distribution"</h2>

            <div class="response-time-chart">
                <div class="chart-bar">
                    <div class="chart-bar-label">
                        <span class="label-text">"< 100ms"</span>
                        <span class="label-badge excellent">"Excellent"</span>
                    </div>
                    <div class="chart-bar-container">
                        <div
                            class="chart-bar-fill excellent"
                            style=format!("width: {}%", (metrics.response_time_bucket_100ms as f64 / total as f64 * 100.0))
                        >
                            <span class="bar-value">{metrics.response_time_bucket_100ms}</span>
                        </div>
                    </div>
                    <span class="chart-percentage">{format!("{:.1}%", metrics.response_time_bucket_100ms as f64 / total as f64 * 100.0)}</span>
                </div>

                <div class="chart-bar">
                    <div class="chart-bar-label">
                        <span class="label-text">"100ms - 500ms"</span>
                        <span class="label-badge good">"Good"</span>
                    </div>
                    <div class="chart-bar-container">
                        <div
                            class="chart-bar-fill good"
                            style=format!("width: {}%", ((metrics.response_time_bucket_500ms - metrics.response_time_bucket_100ms) as f64 / total as f64 * 100.0))
                        >
                            <span class="bar-value">{metrics.response_time_bucket_500ms - metrics.response_time_bucket_100ms}</span>
                        </div>
                    </div>
                    <span class="chart-percentage">{format!("{:.1}%", (metrics.response_time_bucket_500ms - metrics.response_time_bucket_100ms) as f64 / total as f64 * 100.0)}</span>
                </div>

                <div class="chart-bar">
                    <div class="chart-bar-label">
                        <span class="label-text">"500ms - 1s"</span>
                        <span class="label-badge fair">"Fair"</span>
                    </div>
                    <div class="chart-bar-container">
                        <div
                            class="chart-bar-fill fair"
                            style=format!("width: {}%", ((metrics.response_time_bucket_1s - metrics.response_time_bucket_500ms) as f64 / total as f64 * 100.0))
                        >
                            <span class="bar-value">{metrics.response_time_bucket_1s - metrics.response_time_bucket_500ms}</span>
                        </div>
                    </div>
                    <span class="chart-percentage">{format!("{:.1}%", (metrics.response_time_bucket_1s - metrics.response_time_bucket_500ms) as f64 / total as f64 * 100.0)}</span>
                </div>

                <div class="chart-bar">
                    <div class="chart-bar-label">
                        <span class="label-text">"1s - 5s"</span>
                        <span class="label-badge poor">"Poor"</span>
                    </div>
                    <div class="chart-bar-container">
                        <div
                            class="chart-bar-fill poor"
                            style=format!("width: {}%", ((metrics.response_time_bucket_5s - metrics.response_time_bucket_1s) as f64 / total as f64 * 100.0))
                        >
                            <span class="bar-value">{metrics.response_time_bucket_5s - metrics.response_time_bucket_1s}</span>
                        </div>
                    </div>
                    <span class="chart-percentage">{format!("{:.1}%", (metrics.response_time_bucket_5s - metrics.response_time_bucket_1s) as f64 / total as f64 * 100.0)}</span>
                </div>

                <div class="chart-bar">
                    <div class="chart-bar-label">
                        <span class="label-text">"> 5s"</span>
                        <span class="label-badge critical">"Critical"</span>
                    </div>
                    <div class="chart-bar-container">
                        <div
                            class="chart-bar-fill critical"
                            style=format!("width: {}%", (metrics.response_time_bucket_inf as f64 / total as f64 * 100.0))
                        >
                            <span class="bar-value">{metrics.response_time_bucket_inf}</span>
                        </div>
                    </div>
                    <span class="chart-percentage">{format!("{:.1}%", metrics.response_time_bucket_inf as f64 / total as f64 * 100.0)}</span>
                </div>
            </div>

            <div class="performance-insights">
                <div class="insight-card">
                    <div class="insight-icon">"⚡"</div>
                    <div class="insight-content">
                        <div class="insight-title">"Average Response Time"</div>
                        <div class="insight-value">{format!("{:.2}ms", metrics.response_time_avg)}</div>
                        <div class="insight-description">
                            {if metrics.response_time_avg < 100.0 {
                                "Excellent performance! Most requests are served very quickly."
                            } else if metrics.response_time_avg < 500.0 {
                                "Good performance. Consider optimizing slow routes."
                            } else {
                                "Performance needs improvement. Check for bottlenecks."
                            }}
                        </div>
                    </div>
                </div>

                <div class="insight-card">
                    <div class="insight-icon">"📊"</div>
                    <div class="insight-content">
                        <div class="insight-title">"Fast Requests"</div>
                        <div class="insight-value">
                            {format!("{:.1}%", metrics.response_time_bucket_100ms as f64 / total as f64 * 100.0)}
                        </div>
                        <div class="insight-description">
                            "Percentage of requests served in under 100ms"
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// Errors View
// ============================================================================

#[component]
fn ErrorsView(metrics: MetricsData) -> impl IntoView {
    let total_errors = metrics.http_4xx_errors
        + metrics.http_5xx_errors
        + metrics.timeout_errors
        + metrics.connection_errors;

    view! {
        <div class="errors-view">
            <div class="error-summary-cards">
                <div class="error-card client-error">
                    <div class="error-icon">"⚠️"</div>
                    <div class="error-content">
                        <div class="error-value">{metrics.http_4xx_errors}</div>
                        <div class="error-label">"4xx Client Errors"</div>
                        <div class="error-description">"Invalid requests, auth failures, not found"</div>
                    </div>
                </div>

                <div class="error-card server-error">
                    <div class="error-icon">"❌"</div>
                    <div class="error-content">
                        <div class="error-value">{metrics.http_5xx_errors}</div>
                        <div class="error-label">"5xx Server Errors"</div>
                        <div class="error-description">"Backend failures, internal errors"</div>
                    </div>
                </div>

                <div class="error-card timeout-error">
                    <div class="error-icon">"⏱️"</div>
                    <div class="error-content">
                        <div class="error-value">{metrics.timeout_errors}</div>
                        <div class="error-label">"Timeout Errors"</div>
                        <div class="error-description">"Requests that exceeded timeout threshold"</div>
                    </div>
                </div>

                <div class="error-card connection-error">
                    <div class="error-icon">"🔌"</div>
                    <div class="error-content">
                        <div class="error-value">{metrics.connection_errors}</div>
                        <div class="error-label">"Connection Errors"</div>
                        <div class="error-description">"Failed to connect to backends"</div>
                    </div>
                </div>
            </div>

            <div class="error-analysis">
                <h3>"Error Analysis"</h3>

                <div class="analysis-panel">
                    <div class="analysis-metric">
                        <span class="analysis-label">"Total Errors:"</span>
                        <span class="analysis-value">{total_errors}</span>
                    </div>
                    <div class="analysis-metric">
                        <span class="analysis-label">"Error Rate:"</span>
                        <span class="analysis-value">
                            {if metrics.requests_total > 0 {
                                format!("{:.2}%", total_errors as f64 / metrics.requests_total as f64 * 100.0)
                            } else {
                                "0.00%".to_string()
                            }}
                        </span>
                    </div>
                    <div class="analysis-metric">
                        <span class="analysis-label">"Success Rate:"</span>
                        <span class="analysis-value">{format!("{:.2}%", metrics.success_rate)}</span>
                    </div>
                </div>

                <div class="error-recommendations">
                    <h4>"💡 Recommendations"</h4>
                    <ul class="recommendations-list">
                        {if metrics.http_4xx_errors > metrics.requests_total / 10 {
                            view! { <li>"High 4xx error rate detected. Review API documentation and client implementations."</li> }.into_any()
                        } else {
                            view! { <></> }.into_any()
                        }}
                        {if metrics.http_5xx_errors > 0 {
                            view! { <li>"Server errors detected. Check backend health and logs."</li> }.into_any()
                        } else {
                            view! { <></> }.into_any()
                        }}
                        {if metrics.timeout_errors > metrics.requests_total / 20 {
                            view! { <li>"High timeout rate. Consider increasing timeout thresholds or optimizing backend performance."</li> }.into_any()
                        } else {
                            view! { <></> }.into_any()
                        }}
                        {if metrics.connection_errors > 0 {
                            view! { <li>"Connection errors present. Verify backend availability and network connectivity."</li> }.into_any()
                        } else {
                            view! { <></> }.into_any()
                        }}
                        {if total_errors == 0 {
                            view! { <li class="success-message">"✅ No errors detected! All requests are being processed successfully."</li> }.into_any()
                        } else {
                            view! { <></> }.into_any()
                        }}
                    </ul>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// Traffic View
// ============================================================================

#[component]
fn TrafficView(metrics: MetricsData) -> impl IntoView {
    view! {
        <div class="traffic-view">
            <div class="traffic-cards">
                <div class="traffic-card">
                    <div class="traffic-header">
                        <span class="traffic-icon">"📥"</span>
                        <h3>"Incoming Traffic"</h3>
                    </div>
                    <div class="traffic-stats">
                        <div class="stat-large">
                            <div class="stat-value">{MetricsData::format_bytes(metrics.request_bytes_total)}</div>
                            <div class="stat-label">"Total Request Data"</div>
                        </div>
                        <div class="stat-small">
                            <span class="stat-label">"Requests:"</span>
                            <span class="stat-value">{metrics.requests_total}</span>
                        </div>
                        <div class="stat-small">
                            <span class="stat-label">"Avg per request:"</span>
                            <span class="stat-value">
                                {if metrics.requests_total > 0 {
                                    MetricsData::format_bytes(metrics.request_bytes_total / metrics.requests_total)
                                } else {
                                    "0 B".to_string()
                                }}
                            </span>
                        </div>
                    </div>
                </div>

                <div class="traffic-card">
                    <div class="traffic-header">
                        <span class="traffic-icon">"📤"</span>
                        <h3>"Outgoing Traffic"</h3>
                    </div>
                    <div class="traffic-stats">
                        <div class="stat-large">
                            <div class="stat-value">{MetricsData::format_bytes(metrics.response_bytes_total)}</div>
                            <div class="stat-label">"Total Response Data"</div>
                        </div>
                        <div class="stat-small">
                            <span class="stat-label">"Successful:"</span>
                            <span class="stat-value">
                                {format!("{}", (metrics.requests_total as f64 * metrics.success_rate / 100.0) as u64)}
                            </span>
                        </div>
                        <div class="stat-small">
                            <span class="stat-label">"Avg per response:"</span>
                            <span class="stat-value">
                                {if metrics.requests_total > 0 {
                                    MetricsData::format_bytes(metrics.response_bytes_total / metrics.requests_total)
                                } else {
                                    "0 B".to_string()
                                }}
                            </span>
                        </div>
                    </div>
                </div>

                <div class="traffic-card">
                    <div class="traffic-header">
                        <span class="traffic-icon">"🌐"</span>
                        <h3>"Total Bandwidth"</h3>
                    </div>
                    <div class="traffic-stats">
                        <div class="stat-large">
                            <div class="stat-value">{MetricsData::format_bytes(metrics.data_transferred_bytes)}</div>
                            <div class="stat-label">"Total Data Transferred"</div>
                        </div>
                        <div class="stat-small">
                            <span class="stat-label">"In Flight:"</span>
                            <span class="stat-value">{metrics.requests_in_flight}</span>
                        </div>
                        <div class="stat-small">
                            <span class="stat-label">"Active Connections:"</span>
                            <span class="stat-value">{metrics.active_connections}</span>
                        </div>
                    </div>
                </div>
            </div>

            <div class="bandwidth-breakdown">
                <h3>"Bandwidth Breakdown"</h3>
                <div class="bandwidth-chart">
                    <div class="bandwidth-segment">
                        <div class="segment-bar request-bar" style=format!(
                            "width: {}%",
                            if metrics.data_transferred_bytes > 0 {
                                metrics.request_bytes_total as f64 / metrics.data_transferred_bytes as f64 * 100.0
                            } else {
                                0.0
                            }
                        )>
                            <span class="segment-label">"Requests"</span>
                        </div>
                    </div>
                    <div class="bandwidth-segment">
                        <div class="segment-bar response-bar" style=format!(
                            "width: {}%",
                            if metrics.data_transferred_bytes > 0 {
                                metrics.response_bytes_total as f64 / metrics.data_transferred_bytes as f64 * 100.0
                            } else {
                                0.0
                            }
                        )>
                            <span class="segment-label">"Responses"</span>
                        </div>
                    </div>
                </div>

                <div class="bandwidth-legend">
                    <div class="legend-item">
                        <span class="legend-color request-color"></span>
                        <span class="legend-label">
                            {format!("Request Data: {} ({:.1}%)",
                                MetricsData::format_bytes(metrics.request_bytes_total),
                                if metrics.data_transferred_bytes > 0 {
                                    metrics.request_bytes_total as f64 / metrics.data_transferred_bytes as f64 * 100.0
                                } else {
                                    0.0
                                }
                            )}
                        </span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-color response-color"></span>
                        <span class="legend-label">
                            {format!("Response Data: {} ({:.1}%)",
                                MetricsData::format_bytes(metrics.response_bytes_total),
                                if metrics.data_transferred_bytes > 0 {
                                    metrics.response_bytes_total as f64 / metrics.data_transferred_bytes as f64 * 100.0
                                } else {
                                    0.0
                                }
                            )}
                        </span>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// Circuit Breakers View
// ============================================================================

#[component]
fn CircuitBreakersView(metrics: MetricsData) -> impl IntoView {
    let healthy_count = metrics
        .circuit_breakers
        .iter()
        .filter(|cb| cb.state.is_healthy())
        .count();
    let total_count = metrics.circuit_breakers.len();

    view! {
        <div class="circuit-breakers-view">
            <div class="cb-summary">
                <div class="cb-stat">
                    <span class="cb-stat-value">{total_count}</span>
                    <span class="cb-stat-label">"Total Circuit Breakers"</span>
                </div>
                <div class="cb-stat success">
                    <span class="cb-stat-value">{healthy_count}</span>
                    <span class="cb-stat-label">"Healthy (Closed)"</span>
                </div>
                <div class="cb-stat error">
                    <span class="cb-stat-value">{total_count - healthy_count}</span>
                    <span class="cb-stat-label">"Unhealthy (Open/Half-Open)"</span>
                </div>
            </div>

            {if metrics.circuit_breakers.is_empty() {
                view! {
                    <div class="empty-state">
                        <div class="empty-icon">"🔌"</div>
                        <h3>"No Circuit Breakers Configured"</h3>
                        <p>"Circuit breakers protect your gateway from cascading failures."</p>
                        <p>"Configure circuit breakers in your route settings to enable this feature."</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="cb-list">
                        {metrics.circuit_breakers.iter().map(|cb| {
                            let (state_class, state_icon, state_label) = match cb.state {
                                CircuitBreakerState::Closed => ("cb-closed", "✅", "Closed (Healthy)"),
                                CircuitBreakerState::Open => ("cb-open", "❌", "Open (Failed)"),
                                CircuitBreakerState::HalfOpen => ("cb-half-open", "⚠️", "Half-Open (Testing)"),
                            };

                            view! {
                                <div class=format!("cb-item {}", state_class)>
                                    <div class="cb-header">
                                        <div class="cb-route">
                                            <span class="cb-icon">{state_icon}</span>
                                            <span class="cb-route-path">{cb.route.clone()}</span>
                                        </div>
                                        <div class=format!("cb-state {}", state_class)>
                                            {state_label}
                                        </div>
                                    </div>

                                    <div class="cb-stats-grid">
                                        <div class="cb-stat-item success">
                                            <span class="cb-stat-icon">"✅"</span>
                                            <div class="cb-stat-content">
                                                <span class="cb-stat-value">{cb.success_count}</span>
                                                <span class="cb-stat-label">"Successes"</span>
                                            </div>
                                        </div>

                                        <div class="cb-stat-item error">
                                            <span class="cb-stat-icon">"❌"</span>
                                            <div class="cb-stat-content">
                                                <span class="cb-stat-value">{cb.failure_count}</span>
                                                <span class="cb-stat-label">"Failures"</span>
                                            </div>
                                        </div>

                                        {cb.last_failure_time.as_ref().map(|time| view! {
                                            <div class="cb-stat-item">
                                                <span class="cb-stat-icon">"🕐"</span>
                                                <div class="cb-stat-content">
                                                    <span class="cb-stat-value-small">{time.clone()}</span>
                                                    <span class="cb-stat-label">"Last Failure"</span>
                                                </div>
                                            </div>
                                        })}

                                        {cb.next_attempt_time.as_ref().map(|time| view! {
                                            <div class="cb-stat-item">
                                                <span class="cb-stat-icon">"⏰"</span>
                                                <div class="cb-stat-content">
                                                    <span class="cb-stat-value-small">{time.clone()}</span>
                                                    <span class="cb-stat-label">"Next Attempt"</span>
                                                </div>
                                            </div>
                                        })}
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

// ============================================================================
// Historical View
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeRange {
    OneHour,
    SixHours,
    TwentyFourHours,
    SevenDays,
}

impl TimeRange {
    fn duration(&self) -> Duration {
        match self {
            TimeRange::OneHour => Duration::hours(1),
            TimeRange::SixHours => Duration::hours(6),
            TimeRange::TwentyFourHours => Duration::hours(24),
            TimeRange::SevenDays => Duration::days(7),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            TimeRange::OneHour => "1 Hour",
            TimeRange::SixHours => "6 Hours",
            TimeRange::TwentyFourHours => "24 Hours",
            TimeRange::SevenDays => "7 Days",
        }
    }
}

#[component]
fn HistoricalView() -> impl IntoView {
    let (metric_name, set_metric_name) = signal("requests_total".to_string());
    let (interval, set_interval) = signal(AggregationInterval::FiveMinutes);
    let (time_range, set_time_range) = signal(TimeRange::TwentyFourHours);

    let metrics_list_resource = Resource::new(|| (), |_| async move { list_metrics().await });

    let historical_data_resource = Resource::new(
        move || (metric_name.get(), interval.get(), time_range.get()),
        move |(name, interval, range)| async move {
            let end = Utc::now();
            let start = end - range.duration();
            get_historical_metrics(name, start, end, Some(interval)).await
        },
    );

    view! {
        <div class="historical-view">
            <div class="controls-container">
                <div class="control-group">
                    <label>"Metric"</label>
                    <select
                        on:change=move |ev| set_metric_name.set(event_target_value(&ev))
                        prop:value=metric_name
                    >
                        <Suspense fallback=move || view! { <option>"Loading..."</option> }>
                            {move || {
                                metrics_list_resource.get().map(|result| match result {
                                    Ok(list) => list.into_iter().map(|name| view! {
                                        <option value=name.clone()>{name.clone()}</option>
                                    }).collect::<Vec<_>>().into_any(),
                                    Err(_) => view! { <option>"Error loading metrics"</option> }.into_any(),
                                })
                            }}
                        </Suspense>
                    </select>
                </div>

                <div class="control-group">
                    <label>"Aggregation"</label>
                    <select
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            match val.as_str() {
                                "OneMinute" => set_interval.set(AggregationInterval::OneMinute),
                                "FiveMinutes" => set_interval.set(AggregationInterval::FiveMinutes),
                                "OneHour" => set_interval.set(AggregationInterval::OneHour),
                                "OneDay" => set_interval.set(AggregationInterval::OneDay),
                                _ => {}
                            }
                        }
                    >
                        <option value="OneMinute" selected=move || interval.get() == AggregationInterval::OneMinute>"1 Minute"</option>
                        <option value="FiveMinutes" selected=move || interval.get() == AggregationInterval::FiveMinutes>"5 Minutes"</option>
                        <option value="OneHour" selected=move || interval.get() == AggregationInterval::OneHour>"1 Hour"</option>
                        <option value="OneDay" selected=move || interval.get() == AggregationInterval::OneDay>"1 Day"</option>
                    </select>
                </div>

                <div class="control-group time-range-group">
                    <label>"Time Range"</label>
                    <div class="btn-group">
                        {
                            [TimeRange::OneHour, TimeRange::SixHours, TimeRange::TwentyFourHours, TimeRange::SevenDays]
                                .into_iter()
                                .map(|range| {
                                    let label = range.label();
                                    view! {
                                        <button
                                            class=move || if time_range.get() == range { "btn-range active" } else { "btn-range" }
                                            on:click=move |_| set_time_range.set(range)
                                        >
                                            {label}
                                        </button>
                                    }
                                })
                                .collect::<Vec<_>>()
                        }
                    </div>
                </div>
            </div>

            <div class="chart-section">
                <Suspense fallback=move || view! { <LoadingSpinner message="Loading historical data...".to_string() /> }>
                    {move || {
                        historical_data_resource.get().map(|result| match result {
                            Ok(json_data) => {
                                let data: Vec<MetricPoint> = serde_json::from_value(json_data).unwrap_or_default();

                                // Calculate statistics
                                let values: Vec<f64> = data.iter()
                                    .map(|p| match p.value {
                                        MetricValue::Counter(v) => v as f64,
                                        MetricValue::Gauge(v) => v,
                                        MetricValue::Histogram { count, .. } => count as f64,
                                    })
                                    .collect();

                                let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                                let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                                let avg_val = if !values.is_empty() {
                                    values.iter().sum::<f64>() / values.len() as f64
                                } else {
                                    0.0
                                };

                                // Transform data for Chart.js
                                let labels: Vec<String> = data.iter()
                                    .map(|p| p.timestamp.format("%H:%M").to_string())
                                    .collect();

                                let chart_data = serde_json::json!({
                                    "labels": labels,
                                    "datasets": [{
                                        "label": metric_name.get(),
                                        "data": values,
                                        "borderColor": "rgb(99, 102, 241)", // Indigo 500
                                        "backgroundColor": "rgba(99, 102, 241, 0.1)",
                                        "borderWidth": 2,
                                        "pointRadius": 0,
                                        "pointHoverRadius": 4,
                                        "tension": 0.4,
                                        "fill": true
                                    }]
                                });

                                let options = serde_json::json!({
                                    "responsive": true,
                                    "maintainAspectRatio": false,
                                    "interaction": {
                                        "mode": "index",
                                        "intersect": false,
                                    },
                                    "plugins": {
                                        "legend": {
                                            "display": false,
                                        },
                                        "tooltip": {
                                            "backgroundColor": "rgba(17, 24, 39, 0.9)",
                                            "titleColor": "#f3f4f6",
                                            "bodyColor": "#d1d5db",
                                            "borderColor": "#374151",
                                            "borderWidth": 1,
                                            "padding": 10,
                                            "displayColors": false,
                                        }
                                    },
                                    "scales": {
                                        "x": {
                                            "grid": {
                                                "display": false,
                                                "drawBorder": false
                                            },
                                            "ticks": {
                                                "maxTicksLimit": 8,
                                                "color": "#6b7280"
                                            }
                                        },
                                        "y": {
                                            "beginAtZero": true,
                                            "grid": {
                                                "color": "#e5e7eb",
                                                "borderDash": [2, 4],
                                                "drawBorder": false
                                            },
                                            "ticks": {
                                                "color": "#6b7280",
                                                "callback": "function(value) { return value >= 1000 ? (value/1000) + 'k' : value; }"
                                            }
                                        }
                                    }
                                });

                                view! {
                                    <div class="stats-cards">
                                        <div class="stat-card">
                                            <span class="stat-label">"Minimum"</span>
                                            <span class="stat-val">{format!("{:.1}", if min_val == f64::INFINITY { 0.0 } else { min_val })}</span>
                                        </div>
                                        <div class="stat-card">
                                            <span class="stat-label">"Average"</span>
                                            <span class="stat-val">{format!("{:.1}", avg_val)}</span>
                                        </div>
                                        <div class="stat-card">
                                            <span class="stat-label">"Maximum"</span>
                                            <span class="stat-val">{format!("{:.1}", if max_val == f64::NEG_INFINITY { 0.0 } else { max_val })}</span>
                                        </div>
                                    </div>

                                    <div class="chart-container" style="height: 400px; width: 100%;">
                                        <Chart
                                            id="historical-chart"
                                            kind="line"
                                            data=chart_data
                                            options=options
                                        />
                                    </div>
                                }.into_any()
                            }
                            Err(e) => view! {
                                <div class="error-message">
                                    {format!("Error loading data: {}", e)}
                                </div>
                            }.into_any()
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
