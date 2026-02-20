//! User profile page displaying admin info and gateway summary.

use leptos::prelude::*;
use crate::components::{*, StatusVariant};
use crate::server_functions::*;

/// User profile page showing admin details and a gateway summary.
#[component]
pub fn ProfilePage() -> impl IntoView {
    // Fetch health data to show gateway summary
    let health_resource = Resource::new(
        || (),
        |_| async move { get_health().await }
    );

    // Fetch routes to show count
    let routes_resource = Resource::new(
        || (),
        |_| async move { list_routes().await }
    );

    view! {
        <div class="profile-page">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"User Profile"</h1>
                    <p class="page-subtitle">"Admin account details and gateway summary"</p>
                </div>
            </div>

            // Profile card
            <div class="profile-card">
                <div class="profile-avatar">
                    <span class="profile-avatar-icon">"👤"</span>
                </div>
                <div class="profile-info">
                    <h2 class="profile-name">"Gateway Admin"</h2>
                    <p class="profile-role">"Administrator"</p>
                    <div class="profile-badges">
                        <span class="profile-badge profile-badge-admin">"Admin"</span>
                        <span class="profile-badge profile-badge-active">"Active"</span>
                    </div>
                </div>
            </div>

            // Gateway summary section
            <section class="profile-section">
                <h2 class="section-title">"Gateway Summary"</h2>

                <Suspense fallback=move || view! { <LoadingSpinner message="Loading gateway info...".to_string() /> }>
                    {move || {
                        health_resource.get().map(|result| match result {
                            Ok(health) => view! {
                                <div class="profile-details-grid">
                                    <div class="profile-detail-item">
                                        <span class="profile-detail-label">"Status"</span>
                                        <StatusBadge
                                            text=health.status.clone()
                                            variant=if health.is_healthy() { StatusVariant::Success } else { StatusVariant::Error }
                                        />
                                    </div>
                                    <div class="profile-detail-item">
                                        <span class="profile-detail-label">"Version"</span>
                                        <span class="profile-detail-value">{health.version.clone()}</span>
                                    </div>
                                    <div class="profile-detail-item">
                                        <span class="profile-detail-label">"Uptime"</span>
                                        <span class="profile-detail-value">{health.format_uptime()}</span>
                                    </div>
                                    <div class="profile-detail-item">
                                        <span class="profile-detail-label">"Last Check"</span>
                                        <span class="profile-detail-value">{health.timestamp.clone()}</span>
                                    </div>
                                </div>
                            }.into_any(),
                            Err(e) => view! {
                                <ErrorBoundaryView
                                    error=format!("{}", e)
                                    title="Failed to load gateway info".to_string()
                                />
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </section>

            // Routes summary section
            <section class="profile-section">
                <h2 class="section-title">"Routes Overview"</h2>

                <Suspense fallback=move || view! { <LoadingSpinner message="Loading routes...".to_string() /> }>
                    {move || {
                        routes_resource.get().map(|result| match result {
                            Ok(routes) => {
                                let total = routes.len();
                                let auth_count = routes.iter().filter(|r| r.auth_required).count();
                                let public_count = total - auth_count;

                                view! {
                                    <div class="profile-stats-grid">
                                        <div class="profile-stat-card">
                                            <span class="profile-stat-icon">"🛣️"</span>
                                            <span class="profile-stat-value">{total}</span>
                                            <span class="profile-stat-label">"Total Routes"</span>
                                        </div>
                                        <div class="profile-stat-card">
                                            <span class="profile-stat-icon">"🔓"</span>
                                            <span class="profile-stat-value">{public_count}</span>
                                            <span class="profile-stat-label">"Public Routes"</span>
                                        </div>
                                        <div class="profile-stat-card">
                                            <span class="profile-stat-icon">"🔒"</span>
                                            <span class="profile-stat-value">{auth_count}</span>
                                            <span class="profile-stat-label">"Protected Routes"</span>
                                        </div>
                                    </div>
                                }.into_any()
                            },
                            Err(e) => view! {
                                <ErrorBoundaryView
                                    error=format!("{}", e)
                                    title="Failed to load routes".to_string()
                                />
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </section>

            // Quick links section
            <section class="profile-section">
                <h2 class="section-title">"Quick Links"</h2>
                <div class="profile-quick-links">
                    <a href="/routes" class="profile-quick-link">
                        <span class="profile-quick-link-icon">"🛣️"</span>
                        <span>"Manage Routes"</span>
                    </a>
                    <a href="/config" class="profile-quick-link">
                        <span class="profile-quick-link-icon">"⚙️"</span>
                        <span>"Configuration"</span>
                    </a>
                    <a href="/metrics" class="profile-quick-link">
                        <span class="profile-quick-link-icon">"📈"</span>
                        <span>"View Metrics"</span>
                    </a>
                    <a href="/health" class="profile-quick-link">
                        <span class="profile-quick-link-icon">"❤️"</span>
                        <span>"Health Status"</span>
                    </a>
                </div>
            </section>
        </div>
    }
}
