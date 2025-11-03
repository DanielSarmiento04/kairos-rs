//! Configuration management page for gateway settings.
//! 
//! Provides a tabbed interface for managing JWT, rate limiting, CORS,
//! metrics, and server configuration.

use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::models::{
    Settings, JwtSettings, RateLimitConfig, LimitStrategy, WindowType,
    CorsConfig, MetricsConfig, ServerConfig
};
use crate::server_functions::api::{
    get_config, update_jwt_config, update_rate_limit_config,
    update_cors_config, update_metrics_config, update_server_config
};

#[derive(Clone, Copy, PartialEq)]
enum ConfigTab {
    Jwt,
    RateLimit,
    Cors,
    Metrics,
    Server,
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
                                }}
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ============================================================================
// JWT Configuration Form
// ============================================================================

#[component]
fn JwtConfigForm<F>(jwt_settings: Option<JwtSettings>, on_save: F) -> impl IntoView
where
    F: Fn(JwtSettings) + 'static + Clone,
{
    let initial = jwt_settings.unwrap_or_default();
    let (secret, set_secret) = signal(initial.secret.clone());
    let (issuer, set_issuer) = signal(initial.issuer.clone().unwrap_or_default());
    let (audience, set_audience) = signal(initial.audience.clone().unwrap_or_default());
    let (required_claims, set_required_claims) = signal(initial.required_claims.join(", "));
    let (validation_error, set_validation_error) = signal(None::<String>);
    
    let handle_save = move |_| {
        set_validation_error.set(None);
        
        // Validation
        if secret.get().is_empty() {
            set_validation_error.set(Some("Secret cannot be empty".to_string()));
            return;
        }
        
        if secret.get().len() < 32 {
            set_validation_error.set(Some("Secret must be at least 32 characters".to_string()));
            return;
        }
        
        let claims: Vec<String> = required_claims.get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        let config = JwtSettings {
            secret: secret.get(),
            issuer: if issuer.get().is_empty() { None } else { Some(issuer.get()) },
            audience: if audience.get().is_empty() { None } else { Some(audience.get()) },
            required_claims: claims,
        };
        
        on_save(config);
    };
    
    view! {
        <div class="config-form jwt-config">
            <h2>"JWT Authentication Settings"</h2>
            <p class="form-description">
                "Configure JSON Web Token authentication for protected routes."
            </p>
            
            {move || validation_error.get().map(|err| view! {
                <div class="alert alert-error">
                    <span class="alert-icon">"⚠️"</span>
                    <span class="alert-message">{err}</span>
                </div>
            })}
            
            <div class="form-group">
                <label class="form-label required">
                    "Secret Key"
                    <span class="label-hint">"(min 32 characters)"</span>
                </label>
                <input
                    type="password"
                    class="form-input"
                    placeholder="Enter a secure secret key"
                    prop:value=move || secret.get()
                    on:input=move |ev| set_secret.set(event_target_value(&ev))
                />
                <p class="form-help">
                    "Strong secret key for signing JWT tokens. Must be at least 32 characters."
                </p>
            </div>
            
            <div class="form-group">
                <label class="form-label">
                    "Issuer"
                    <span class="label-hint">"(optional)"</span>
                </label>
                <input
                    type="text"
                    class="form-input"
                    placeholder="e.g., kairos-gateway"
                    prop:value=move || issuer.get()
                    on:input=move |ev| set_issuer.set(event_target_value(&ev))
                />
                <p class="form-help">
                    "Expected issuer (iss claim) for JWT validation."
                </p>
            </div>
            
            <div class="form-group">
                <label class="form-label">
                    "Audience"
                    <span class="label-hint">"(optional)"</span>
                </label>
                <input
                    type="text"
                    class="form-input"
                    placeholder="e.g., api-clients"
                    prop:value=move || audience.get()
                    on:input=move |ev| set_audience.set(event_target_value(&ev))
                />
                <p class="form-help">
                    "Expected audience (aud claim) for JWT validation."
                </p>
            </div>
            
            <div class="form-group">
                <label class="form-label">
                    "Required Claims"
                    <span class="label-hint">"(comma-separated)"</span>
                </label>
                <input
                    type="text"
                    class="form-input"
                    placeholder="sub, exp, iat"
                    prop:value=move || required_claims.get()
                    on:input=move |ev| set_required_claims.set(event_target_value(&ev))
                />
                <p class="form-help">
                    "Comma-separated list of claims that must be present in valid tokens."
                </p>
            </div>
            
            <div class="form-actions">
                <button class="btn btn-primary" on:click=handle_save>
                    "💾 Save JWT Configuration"
                </button>
            </div>
        </div>
    }
}

// ============================================================================
// Rate Limit Configuration Form
// ============================================================================

#[component]
fn RateLimitConfigForm<F>(rate_limit: Option<RateLimitConfig>, on_save: F) -> impl IntoView
where
    F: Fn(RateLimitConfig) + 'static + Clone,
{
    let initial = rate_limit.unwrap_or_default();
    let (strategy, set_strategy) = signal(initial.strategy);
    let (requests_per_window, set_requests_per_window) = signal(initial.requests_per_window.to_string());
    let (window_duration, set_window_duration) = signal(initial.window_duration_secs.to_string());
    let (burst_allowance, set_burst_allowance) = signal(initial.burst_allowance.to_string());
    let (window_type, set_window_type) = signal(initial.window_type);
    let (enable_redis, set_enable_redis) = signal(initial.enable_redis);
    let (redis_prefix, set_redis_prefix) = signal(initial.redis_key_prefix);
    let (validation_error, set_validation_error) = signal(None::<String>);
    
    let handle_save = move |_| {
        set_validation_error.set(None);
        
        let requests: u64 = match requests_per_window.get().parse() {
            Ok(v) if v > 0 => v,
            _ => {
                set_validation_error.set(Some("Requests per window must be a positive number".to_string()));
                return;
            }
        };
        
        let duration: u64 = match window_duration.get().parse() {
            Ok(v) if v > 0 => v,
            _ => {
                set_validation_error.set(Some("Window duration must be a positive number".to_string()));
                return;
            }
        };
        
        let burst: u64 = match burst_allowance.get().parse() {
            Ok(v) => v,
            _ => {
                set_validation_error.set(Some("Burst allowance must be a valid number".to_string()));
                return;
            }
        };
        
        let config = RateLimitConfig {
            strategy: strategy.get(),
            requests_per_window: requests,
            window_duration_secs: duration,
            burst_allowance: burst,
            window_type: window_type.get(),
            enable_redis: enable_redis.get(),
            redis_key_prefix: redis_prefix.get(),
        };
        
        on_save(config);
    };
    
    view! {
        <div class="config-form rate-limit-config">
            <h2>"Rate Limiting Configuration"</h2>
            <p class="form-description">
                "Configure rate limiting to protect your gateway from abuse and ensure fair usage."
            </p>
            
            {move || validation_error.get().map(|err| view! {
                <div class="alert alert-error">
                    <span class="alert-icon">"⚠️"</span>
                    <span class="alert-message">{err}</span>
                </div>
            })}
            
            <div class="form-row">
                <div class="form-group">
                    <label class="form-label required">"Strategy"</label>
                    <select
                        class="form-select"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_strategy.set(match value.as_str() {
                                "PerIP" => LimitStrategy::PerIP,
                                "PerUser" => LimitStrategy::PerUser,
                                "PerRoute" => LimitStrategy::PerRoute,
                                "PerIPAndRoute" => LimitStrategy::PerIPAndRoute,
                                "PerUserAndRoute" => LimitStrategy::PerUserAndRoute,
                                _ => LimitStrategy::PerIP,
                            });
                        }
                    >
                        <option value="PerIP" selected=move || matches!(strategy.get(), LimitStrategy::PerIP)>"Per IP"</option>
                        <option value="PerUser" selected=move || matches!(strategy.get(), LimitStrategy::PerUser)>"Per User"</option>
                        <option value="PerRoute" selected=move || matches!(strategy.get(), LimitStrategy::PerRoute)>"Per Route"</option>
                        <option value="PerIPAndRoute" selected=move || matches!(strategy.get(), LimitStrategy::PerIPAndRoute)>"Per IP + Route"</option>
                        <option value="PerUserAndRoute" selected=move || matches!(strategy.get(), LimitStrategy::PerUserAndRoute)>"Per User + Route"</option>
                    </select>
                </div>
                
                <div class="form-group">
                    <label class="form-label required">"Window Type"</label>
                    <select
                        class="form-select"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_window_type.set(match value.as_str() {
                                "FixedWindow" => WindowType::FixedWindow,
                                "SlidingWindow" => WindowType::SlidingWindow,
                                "TokenBucket" => WindowType::TokenBucket,
                                _ => WindowType::SlidingWindow,
                            });
                        }
                    >
                        <option value="FixedWindow" selected=move || matches!(window_type.get(), WindowType::FixedWindow)>"Fixed Window"</option>
                        <option value="SlidingWindow" selected=move || matches!(window_type.get(), WindowType::SlidingWindow)>"Sliding Window"</option>
                        <option value="TokenBucket" selected=move || matches!(window_type.get(), WindowType::TokenBucket)>"Token Bucket"</option>
                    </select>
                </div>
            </div>
            
            <div class="form-row">
                <div class="form-group">
                    <label class="form-label required">"Requests per Window"</label>
                    <input
                        type="number"
                        class="form-input"
                        min="1"
                        placeholder="100"
                        prop:value=move || requests_per_window.get()
                        on:input=move |ev| set_requests_per_window.set(event_target_value(&ev))
                    />
                </div>
                
                <div class="form-group">
                    <label class="form-label required">"Window Duration (seconds)"</label>
                    <input
                        type="number"
                        class="form-input"
                        min="1"
                        placeholder="60"
                        prop:value=move || window_duration.get()
                        on:input=move |ev| set_window_duration.set(event_target_value(&ev))
                    />
                </div>
                
                <div class="form-group">
                    <label class="form-label">"Burst Allowance"</label>
                    <input
                        type="number"
                        class="form-input"
                        min="0"
                        placeholder="20"
                        prop:value=move || burst_allowance.get()
                        on:input=move |ev| set_burst_allowance.set(event_target_value(&ev))
                    />
                </div>
            </div>
            
            <div class="form-group">
                <label class="form-checkbox">
                    <input
                        type="checkbox"
                        prop:checked=move || enable_redis.get()
                        on:change=move |ev| set_enable_redis.set(event_target_checked(&ev))
                    />
                    <span>"Enable Redis for distributed rate limiting"</span>
                </label>
            </div>
            
            {move || enable_redis.get().then(|| view! {
                <div class="form-group">
                    <label class="form-label">"Redis Key Prefix"</label>
                    <input
                        type="text"
                        class="form-input"
                        placeholder="kairos_rl"
                        prop:value=move || redis_prefix.get()
                        on:input=move |ev| set_redis_prefix.set(event_target_value(&ev))
                    />
                </div>
            })}
            
            <div class="form-actions">
                <button class="btn btn-primary" on:click=handle_save>
                    "💾 Save Rate Limit Configuration"
                </button>
            </div>
        </div>
    }
}

// ============================================================================
// CORS Configuration Form
// ============================================================================

#[component]
fn CorsConfigForm<F>(cors_config: Option<CorsConfig>, on_save: F) -> impl IntoView
where
    F: Fn(CorsConfig) + 'static + Clone,
{
    let initial = cors_config.unwrap_or_default();
    let (enabled, set_enabled) = signal(initial.enabled);
    let (allowed_origins, set_allowed_origins) = signal(initial.allowed_origins.join("\n"));
    let (allowed_methods, set_allowed_methods) = signal(initial.allowed_methods.join(", "));
    let (allowed_headers, set_allowed_headers) = signal(initial.allowed_headers.join(", "));
    let (allow_credentials, set_allow_credentials) = signal(initial.allow_credentials);
    let (max_age, set_max_age) = signal(initial.max_age_secs.map(|v| v.to_string()).unwrap_or_default());
    
    let handle_save = move |_| {
        let origins: Vec<String> = allowed_origins.get()
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        let methods: Vec<String> = allowed_methods.get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        let headers: Vec<String> = allowed_headers.get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        let max_age_secs = if max_age.get().is_empty() {
            None
        } else {
            max_age.get().parse().ok()
        };
        
        let config = CorsConfig {
            enabled: enabled.get(),
            allowed_origins: origins,
            allowed_methods: methods,
            allowed_headers: headers,
            allow_credentials: allow_credentials.get(),
            max_age_secs,
        };
        
        on_save(config);
    };
    
    view! {
        <div class="config-form cors-config">
            <h2>"CORS Configuration"</h2>
            <p class="form-description">
                "Configure Cross-Origin Resource Sharing (CORS) to control which domains can access your API."
            </p>
            
            <div class="form-group">
                <label class="form-checkbox">
                    <input
                        type="checkbox"
                        prop:checked=move || enabled.get()
                        on:change=move |ev| set_enabled.set(event_target_checked(&ev))
                    />
                    <span>"Enable CORS"</span>
                </label>
            </div>
            
            {move || enabled.get().then(|| view! {
                <>
                    <div class="form-group">
                        <label class="form-label">"Allowed Origins"<span class="label-hint">" (one per line)"</span></label>
                        <textarea
                            class="form-textarea"
                            rows="4"
                            placeholder="https://example.com\nhttps://app.example.com\n*"
                            prop:value=move || allowed_origins.get()
                            on:input=move |ev| set_allowed_origins.set(event_target_value(&ev))
                        ></textarea>
                        <p class="form-help">
                            "Specify allowed origins. Use * to allow all origins (not recommended for production)."
                        </p>
                    </div>
                    
                    <div class="form-group">
                        <label class="form-label">"Allowed Methods"<span class="label-hint">" (comma-separated)"</span></label>
                        <input
                            type="text"
                            class="form-input"
                            placeholder="GET, POST, PUT, DELETE, OPTIONS"
                            prop:value=move || allowed_methods.get()
                            on:input=move |ev| set_allowed_methods.set(event_target_value(&ev))
                        />
                    </div>
                    
                    <div class="form-group">
                        <label class="form-label">"Allowed Headers"<span class="label-hint">" (comma-separated)"</span></label>
                        <input
                            type="text"
                            class="form-input"
                            placeholder="Content-Type, Authorization, X-Custom-Header"
                            prop:value=move || allowed_headers.get()
                            on:input=move |ev| set_allowed_headers.set(event_target_value(&ev))
                        />
                    </div>
                    
                    <div class="form-row">
                        <div class="form-group">
                            <label class="form-checkbox">
                                <input
                                    type="checkbox"
                                    prop:checked=move || allow_credentials.get()
                                    on:change=move |ev| set_allow_credentials.set(event_target_checked(&ev))
                                />
                                <span>"Allow Credentials (cookies, auth headers)"</span>
                            </label>
                        </div>
                        
                        <div class="form-group">
                            <label class="form-label">"Max Age (seconds)"</label>
                            <input
                                type="number"
                                class="form-input"
                                min="0"
                                placeholder="3600"
                                prop:value=move || max_age.get()
                                on:input=move |ev| set_max_age.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                </>
            })}
            
            <div class="form-actions">
                <button class="btn btn-primary" on:click=handle_save>
                    "💾 Save CORS Configuration"
                </button>
            </div>
        </div>
    }
}

// ============================================================================
// Metrics Configuration Form
// ============================================================================

#[component]
fn MetricsConfigForm<F>(metrics_config: Option<MetricsConfig>, on_save: F) -> impl IntoView
where
    F: Fn(MetricsConfig) + 'static + Clone,
{
    let initial = metrics_config.unwrap_or_default();
    let (enabled, set_enabled) = signal(initial.enabled);
    let (endpoint, set_endpoint) = signal(initial.prometheus_endpoint);
    let (collect_per_route, set_collect_per_route) = signal(initial.collect_per_route);
    
    let handle_save = move |_| {
        let config = MetricsConfig {
            enabled: enabled.get(),
            prometheus_endpoint: endpoint.get(),
            collect_per_route: collect_per_route.get(),
        };
        
        on_save(config);
    };
    
    view! {
        <div class="config-form metrics-config">
            <h2>"Metrics Configuration"</h2>
            <p class="form-description">
                "Configure Prometheus metrics collection and exposure."
            </p>
            
            <div class="form-group">
                <label class="form-checkbox">
                    <input
                        type="checkbox"
                        prop:checked=move || enabled.get()
                        on:change=move |ev| set_enabled.set(event_target_checked(&ev))
                    />
                    <span>"Enable Metrics Collection"</span>
                </label>
            </div>
            
            {move || enabled.get().then(|| view! {
                <>
                    <div class="form-group">
                        <label class="form-label">"Prometheus Endpoint"</label>
                        <input
                            type="text"
                            class="form-input"
                            placeholder="/metrics"
                            prop:value=move || endpoint.get()
                            on:input=move |ev| set_endpoint.set(event_target_value(&ev))
                        />
                        <p class="form-help">
                            "Path where Prometheus metrics will be exposed."
                        </p>
                    </div>
                    
                    <div class="form-group">
                        <label class="form-checkbox">
                            <input
                                type="checkbox"
                                prop:checked=move || collect_per_route.get()
                                on:change=move |ev| set_collect_per_route.set(event_target_checked(&ev))
                            />
                            <span>"Collect Per-Route Metrics"</span>
                        </label>
                        <p class="form-help">
                            "Track metrics individually for each route (increases cardinality)."
                        </p>
                    </div>
                </>
            })}
            
            <div class="form-actions">
                <button class="btn btn-primary" on:click=handle_save>
                    "💾 Save Metrics Configuration"
                </button>
            </div>
        </div>
    }
}

// ============================================================================
// Server Configuration Form
// ============================================================================

#[component]
fn ServerConfigForm<F>(server_config: Option<ServerConfig>, on_save: F) -> impl IntoView
where
    F: Fn(ServerConfig) + 'static + Clone,
{
    let initial = server_config.unwrap_or_default();
    let (host, set_host) = signal(initial.host);
    let (port, set_port) = signal(initial.port.to_string());
    let (workers, set_workers) = signal(initial.workers.to_string());
    let (keep_alive, set_keep_alive) = signal(initial.keep_alive_secs.to_string());
    let (validation_error, set_validation_error) = signal(None::<String>);
    
    let handle_save = move |_| {
        set_validation_error.set(None);
        
        let port_num: u16 = match port.get().parse() {
            Ok(v) if v > 0 => v,
            _ => {
                set_validation_error.set(Some("Port must be a valid number between 1-65535".to_string()));
                return;
            }
        };
        
        let workers_num: usize = match workers.get().parse() {
            Ok(v) if v > 0 => v,
            _ => {
                set_validation_error.set(Some("Workers must be a positive number".to_string()));
                return;
            }
        };
        
        let keep_alive_num: u64 = match keep_alive.get().parse() {
            Ok(v) => v,
            _ => {
                set_validation_error.set(Some("Keep-alive must be a valid number".to_string()));
                return;
            }
        };
        
        let config = ServerConfig {
            host: host.get(),
            port: port_num,
            workers: workers_num,
            keep_alive_secs: keep_alive_num,
        };
        
        on_save(config);
    };
    
    view! {
        <div class="config-form server-config">
            <h2>"Server Configuration"</h2>
            <p class="form-description">
                "Configure server runtime settings and performance tuning."
            </p>
            
            {move || validation_error.get().map(|err| view! {
                <div class="alert alert-error">
                    <span class="alert-icon">"⚠️"</span>
                    <span class="alert-message">{err}</span>
                </div>
            })}
            
            <div class="form-row">
                <div class="form-group">
                    <label class="form-label required">"Host"</label>
                    <input
                        type="text"
                        class="form-input"
                        placeholder="0.0.0.0"
                        prop:value=move || host.get()
                        on:input=move |ev| set_host.set(event_target_value(&ev))
                    />
                    <p class="form-help">
                        "Bind address (0.0.0.0 for all interfaces, 127.0.0.1 for localhost only)."
                    </p>
                </div>
                
                <div class="form-group">
                    <label class="form-label required">"Port"</label>
                    <input
                        type="number"
                        class="form-input"
                        min="1"
                        max="65535"
                        placeholder="5900"
                        prop:value=move || port.get()
                        on:input=move |ev| set_port.set(event_target_value(&ev))
                    />
                </div>
            </div>
            
            <div class="form-row">
                <div class="form-group">
                    <label class="form-label required">"Worker Threads"</label>
                    <input
                        type="number"
                        class="form-input"
                        min="1"
                        placeholder="4"
                        prop:value=move || workers.get()
                        on:input=move |ev| set_workers.set(event_target_value(&ev))
                    />
                    <p class="form-help">
                        "Number of worker threads (typically CPU cores)."
                    </p>
                </div>
                
                <div class="form-group">
                    <label class="form-label">"Keep-Alive (seconds)"</label>
                    <input
                        type="number"
                        class="form-input"
                        min="0"
                        placeholder="75"
                        prop:value=move || keep_alive.get()
                        on:input=move |ev| set_keep_alive.set(event_target_value(&ev))
                    />
                    <p class="form-help">
                        "HTTP keep-alive timeout (0 to disable)."
                    </p>
                </div>
            </div>
            
            <div class="form-actions">
                <button class="btn btn-primary" on:click=handle_save>
                    "💾 Save Server Configuration"
                </button>
            </div>
        </div>
    }
}
