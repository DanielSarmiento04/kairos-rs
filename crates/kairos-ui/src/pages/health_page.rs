//! Health monitoring page for system status.

use crate::components::{StatusVariant, *};
use crate::server_functions::*;
use leptos::prelude::*;

/// Health monitoring page with detailed status checks.
#[component]
pub fn HealthPage() -> impl IntoView {
    // Fetch health data
    let health_resource = Resource::new(|| (), |_| async move { get_health().await });

    let readiness_resource = Resource::new(|| (), |_| async move { get_readiness().await });

    let liveness_resource = Resource::new(|| (), |_| async move { get_liveness().await });

    view! {
        <div class="health-page">
            <div class="page-header">
                <h1 class="page-title">"Health Monitoring"</h1>
                <p class="page-subtitle">"System health status and diagnostics"</p>
            </div>

            <section class="health-section">
                <h2 class="section-title">"General Health"</h2>

                <Suspense fallback=move || view! { <LoadingSpinner message="Loading health status...".to_string() /> }>
                    {move || {
                        health_resource.get().map(|result| match result {
                            Ok(health) => view! {
                                <div class="health-details">
                                    <div class="health-item">
                                        <span class="health-label">"Status:"</span>
                                        <StatusBadge
                                            text=health.status.clone()
                                            variant=if health.is_healthy() { StatusVariant::Success } else { StatusVariant::Error }
                                        />
                                    </div>

                                    <div class="health-item">
                                        <span class="health-label">"Version:"</span>
                                        <span class="health-value">{health.version.clone()}</span>
                                    </div>

                                    <div class="health-item">
                                        <span class="health-label">"Uptime:"</span>
                                        <span class="health-value">{health.format_uptime()}</span>
                                    </div>

                                    <div class="health-item">
                                        <span class="health-label">"Last Check:"</span>
                                        <span class="health-value">{health.timestamp.clone()}</span>
                                    </div>
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

            <section class="health-section">
                <h2 class="section-title">"Readiness Probe"</h2>

                <Suspense fallback=move || view! { <LoadingSpinner /> }>
                    {move || {
                        readiness_resource.get().map(|result| match result {
                            Ok(ready) => view! {
                                <div class="health-details">
                                    <div class="health-item">
                                        <span class="health-label">"Status:"</span>
                                        <StatusBadge
                                            text=ready.status.clone()
                                            variant=if ready.is_ready() { StatusVariant::Success } else { StatusVariant::Warning }
                                        />
                                    </div>

                                    <div class="health-item">
                                        <span class="health-label">"Timestamp:"</span>
                                        <span class="health-value">{ready.timestamp.clone()}</span>
                                    </div>
                                </div>
                            }.into_any(),
                            Err(e) => view! {
                                <ErrorBoundaryView error=format!("{}", e) />
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </section>

            <section class="health-section">
                <h2 class="section-title">"Liveness Probe"</h2>

                <Suspense fallback=move || view! { <LoadingSpinner /> }>
                    {move || {
                        liveness_resource.get().map(|result| match result {
                            Ok(live) => view! {
                                <div class="health-details">
                                    <div class="health-item">
                                        <span class="health-label">"Status:"</span>
                                        <StatusBadge
                                            text=live.status.clone()
                                            variant=if live.is_alive() { StatusVariant::Success } else { StatusVariant::Error }
                                        />
                                    </div>

                                    <div class="health-item">
                                        <span class="health-label">"Timestamp:"</span>
                                        <span class="health-value">{live.timestamp.clone()}</span>
                                    </div>
                                </div>
                            }.into_any(),
                            Err(e) => view! {
                                <ErrorBoundaryView error=format!("{}", e) />
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </section>
        </div>
    }
}
