//! Configuration management page for gateway settings.
//!
//! Provides a tabbed interface for managing JWT, rate limiting, CORS,
//! metrics, and server configuration.

use crate::components::config::*;
use crate::models::{
    AiSettings, CorsConfig, JwtSettings, LimitStrategy, MetricsConfig, RateLimitConfig,
    ServerConfig, Settings, WindowType,
};
use crate::server_functions::{
    get_config, update_ai_config, update_cors_config, update_jwt_config, update_metrics_config,
    update_rate_limit_config, update_server_config,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[derive(Clone, Copy, PartialEq)]
enum ConfigTab {
    Jwt,
    RateLimit,
    Cors,
    Metrics,
    Server,
    Ai,
}

/// Main configuration page component with tabbed interface.
#[component]
pub fn ConfigPage() -> impl IntoView {
    let (active_tab, set_active_tab) = signal(ConfigTab::Jwt);
    let (settings, set_settings) = signal(Settings::default());
    let (loading, set_loading) = signal(true);
    let (error_message, set_error_message) = signal(None::<String>);
    let (success_message, set_success_message) = signal(None::<String>);

    // Load configuration on mount
    Effect::new(move || {
        spawn_local(async move {
            set_loading.set(true);
            set_error_message.set(None);

            match get_config().await {
                Ok(config) => {
                    set_settings.set(config);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error_message.set(Some(format!("Failed to load configuration: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    });

    view! {
        <div class="config-page">
            <div class="page-header">
                <h1 class="page-title">"⚙️ Configuration"</h1>
                <p class="page-subtitle">"Manage gateway configuration and settings"</p>
            </div>

            {move || error_message.get().map(|msg| view! {
                <div class="alert alert-error">
                    <span class="alert-icon">"⚠️"</span>
                    <span class="alert-message">{msg}</span>
                </div>
            })}

            {move || success_message.get().map(|msg| view! {
                <div class="alert alert-success">
                    <span class="alert-icon">"✅"</span>
                    <span class="alert-message">{msg}</span>
                </div>
            })}

            {move || {
                if loading.get() {
                    view! {
                        <div class="loading-container">
                            <div class="spinner"></div>
                            <p>"Loading configuration..."</p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="config-tabs-container">
                            <div class="tabs-nav">
                                <button
                                    class=move || if active_tab.get() == ConfigTab::Jwt { "tab-button active" } else { "tab-button" }
                                    on:click=move |_| set_active_tab.set(ConfigTab::Jwt)
                                >
                                    "🔐 JWT Authentication"
                                </button>
                                <button
                                    class=move || if active_tab.get() == ConfigTab::RateLimit { "tab-button active" } else { "tab-button" }
                                    on:click=move |_| set_active_tab.set(ConfigTab::RateLimit)
                                >
                                    "⏱️ Rate Limiting"
                                </button>
                                <button
                                    class=move || if active_tab.get() == ConfigTab::Cors { "tab-button active" } else { "tab-button" }
                                    on:click=move |_| set_active_tab.set(ConfigTab::Cors)
                                >
                                    "🌐 CORS"
                                </button>
                                <button
                                    class=move || if active_tab.get() == ConfigTab::Metrics { "tab-button active" } else { "tab-button" }
                                    on:click=move |_| set_active_tab.set(ConfigTab::Metrics)
                                >
                                    "📊 Metrics"
                                </button>
                                <button
                                    class=move || if active_tab.get() == ConfigTab::Server { "tab-button active" } else { "tab-button" }
                                    on:click=move |_| set_active_tab.set(ConfigTab::Server)
                                >
                                    "🖥️ Server"
                                </button>
                                <button
                                    class=move || if active_tab.get() == ConfigTab::Ai { "tab-button active" } else { "tab-button" }
                                    on:click=move |_| set_active_tab.set(ConfigTab::Ai)
                                >
                                    "🤖 AI Configuration"
                                </button>
                            </div>

                            <div class="tab-content">
                                {move || match active_tab.get() {
                                    ConfigTab::Jwt => view! {
                                        <JwtConfigForm
                                            jwt_settings=settings.get().jwt
                                            on_save=move |config| {
                                                let set_success = set_success_message.clone();
                                                let set_error = set_error_message.clone();
                                                spawn_local(async move {
                                                    set_error.set(None);
                                                    match update_jwt_config(config).await {
                                                        Ok(_) => set_success.set(Some("JWT configuration updated successfully!".to_string())),
                                                        Err(e) => set_error.set(Some(format!("Failed to update JWT config: {}", e))),
                                                    }
                                                });
                                            }
                                        />
                                    }.into_any(),
                                    ConfigTab::RateLimit => view! {
                                        <RateLimitConfigForm
                                            rate_limit=settings.get().rate_limit
                                            on_save=move |config| {
                                                let set_success = set_success_message.clone();
                                                let set_error = set_error_message.clone();
                                                spawn_local(async move {
                                                    set_error.set(None);
                                                    match update_rate_limit_config(config).await {
                                                        Ok(_) => set_success.set(Some("Rate limiting configuration updated successfully!".to_string())),
                                                        Err(e) => set_error.set(Some(format!("Failed to update rate limit config: {}", e))),
                                                    }
                                                });
                                            }
                                        />
                                    }.into_any(),
                                    ConfigTab::Cors => view! {
                                        <CorsConfigForm
                                            cors_config=settings.get().cors
                                            on_save=move |config| {
                                                let set_success = set_success_message.clone();
                                                let set_error = set_error_message.clone();
                                                spawn_local(async move {
                                                    set_error.set(None);
                                                    match update_cors_config(config).await {
                                                        Ok(_) => set_success.set(Some("CORS configuration updated successfully!".to_string())),
                                                        Err(e) => set_error.set(Some(format!("Failed to update CORS config: {}", e))),
                                                    }
                                                });
                                            }
                                        />
                                    }.into_any(),
                                    ConfigTab::Metrics => view! {
                                        <MetricsConfigForm
                                            metrics_config=settings.get().metrics
                                            on_save=move |config| {
                                                let set_success = set_success_message.clone();
                                                let set_error = set_error_message.clone();
                                                spawn_local(async move {
                                                    set_error.set(None);
                                                    match update_metrics_config(config).await {
                                                        Ok(_) => set_success.set(Some("Metrics configuration updated successfully!".to_string())),
                                                        Err(e) => set_error.set(Some(format!("Failed to update metrics config: {}", e))),
                                                    }
                                                });
                                            }
                                        />
                                    }.into_any(),
                                    ConfigTab::Server => view! {
                                        <ServerConfigForm
                                            server_config=settings.get().server
                                            on_save=move |config| {
                                                let set_success = set_success_message.clone();
                                                let set_error = set_error_message.clone();
                                                spawn_local(async move {
                                                    set_error.set(None);
                                                    match update_server_config(config).await {
                                                        Ok(_) => set_success.set(Some("Server configuration updated successfully!".to_string())),
                                                        Err(e) => set_error.set(Some(format!("Failed to update server config: {}", e))),
                                                    }
                                                });
                                            }
                                        />
                                    }.into_any(),
                                    ConfigTab::Ai => view! {
                                        <AiConfigForm
                                            ai_settings=settings.get().ai
                                            on_save=move |config| {
                                                let set_success = set_success_message.clone();
                                                let set_error = set_error_message.clone();
                                                spawn_local(async move {
                                                    set_error.set(None);
                                                    match update_ai_config(config).await {
                                                        Ok(_) => set_success.set(Some("AI configuration updated successfully!".to_string())),
                                                        Err(e) => set_error.set(Some(format!("Failed to update AI config: {}", e))),
                                                    }
                                                });
                                            }
                                        />
                                    }.into_any(),
                                }}
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
