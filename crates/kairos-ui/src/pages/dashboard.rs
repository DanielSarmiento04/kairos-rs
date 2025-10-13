//! Dashboard page with real-time metrics and system status.

use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::components::{*, StatusVariant};
use crate::server_functions::*;
use crate::models::*;

/// Main dashboard page displaying gateway health and metrics.
#[component]
pub fn DashboardPage() -> impl IntoView {
    // Auto-refresh signal (triggers every 30 seconds)
    let refresh_trigger = RwSignal::new(0u32);
    
    // Set up auto-refresh timer
    spawn_local(async move {
        loop {
            #[cfg(feature = "hydrate")]
            {
                gloo_timers::future::TimeoutFuture::new(30_000).await;
                refresh_trigger.update(|n| *n += 1);
            }
            #[cfg(not(feature = "hydrate"))]
            {
                // On server, just break to avoid infinite loop
                break;
            }
        }
    });
    
    // Fetch health data
    let health_resource = Resource::new(
        move || refresh_trigger.get(),
        |_| async move { get_health().await }
    );
    
    // Fetch metrics data
    let metrics_resource = Resource::new(
        move || refresh_trigger.get(),
        |_| async move { get_metrics().await }
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
                                        icon=(if health.is_healthy() { "✅" } else { "❌" }.to_string())
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
                            Ok(metrics) => view! {
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
                                        trend=(if metrics.success_rate >= 95.0 { "up" } else if metrics.success_rate >= 80.0 { "neutral" } else { "down" }.to_string())
                                    />
                                    
                                    <MetricCard 
                                        title="Avg Response Time".to_string()
                                        value=format!("{:.2}ms", metrics.response_time_avg)
                                        icon="⚡".to_string()
                                        trend=(if metrics.response_time_avg < 100.0 { "up" } else if metrics.response_time_avg < 500.0 { "neutral" } else { "down" }.to_string())
                                    />
                                    
                                    <MetricCard 
                                        title="Active Connections".to_string()
                                        value=format!("{}", metrics.active_connections)
                                        icon="🔗".to_string()
                                        subtitle=(format!("Peak: {}", metrics.peak_connections))
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
                            }.into_any(),
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
