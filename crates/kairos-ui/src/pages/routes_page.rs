//! Routes management page for viewing and editing gateway routes.
//!
//! This page provides a comprehensive CRUD interface for managing gateway routes,
//! including creating, updating, deleting, and testing route configurations.

use crate::components::*;
use crate::models::router::*;
use crate::server_functions::*;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Routes management page with full CRUD functionality.
#[component]
pub fn RoutesPage() -> impl IntoView {
    // State for form visibility
    let (show_form, set_show_form) = signal(false);
    let (editing_route, set_editing_route) = signal::<Option<Router>>(None);
    let (form_error, set_form_error) = signal::<Option<String>>(None);
    let (form_success, set_form_success) = signal::<Option<String>>(None);

    // Reload trigger
    let (reload_trigger, set_reload_trigger) = signal(0);

    // Create a resource that responds to the reload trigger
    let routes_resource = Resource::new(
        move || reload_trigger.get(),
        |_| async move { list_routes().await },
    );

    // Handler for creating a new route
    let on_add_click = move |_| {
        set_editing_route.set(None);
        set_show_form.set(true);
        set_form_error.set(None);
        set_form_success.set(None);
    };

    // Handler for editing a route
    let on_edit = move |route: Router| {
        set_editing_route.set(Some(route));
        set_show_form.set(true);
        set_form_error.set(None);
        set_form_success.set(None);
    };

    // Handler for deleting a route
    let on_delete = move |external_path: String| {
        spawn_local(async move {
            match delete_route(external_path.clone()).await {
                Ok(_) => {
                    set_form_success.set(Some(format!(
                        "Route '{}' deleted successfully!",
                        external_path
                    )));
                    set_reload_trigger.update(|n| *n += 1);
                }
                Err(e) => {
                    set_form_error.set(Some(format!("Failed to delete route: {}", e)));
                }
            }
        });
    };

    // Handler for form submission
    let on_submit = move |route: Router| {
        let is_editing = editing_route.get().is_some();

        spawn_local(async move {
            let result = if is_editing {
                update_route(route.clone()).await
            } else {
                create_route(route.clone()).await
            };

            match result {
                Ok(_) => {
                    let action = if is_editing { "updated" } else { "created" };
                    set_form_success.set(Some(format!(
                        "Route '{}' {} successfully!",
                        route.external_path, action
                    )));
                    set_show_form.set(false);
                    set_editing_route.set(None);
                    set_reload_trigger.update(|n| *n += 1);
                }
                Err(e) => {
                    set_form_error.set(Some(format!("Failed to save route: {}", e)));
                }
            }
        });
    };

    // Handler for form cancel
    let on_cancel = move |_| {
        set_show_form.set(false);
        set_editing_route.set(None);
        set_form_error.set(None);
    };

    view! {
        <div class="routes-page">
            <div class="page-header">
                <h1 class="page-title">"Routes Management"</h1>
                <p class="page-subtitle">"Configure and manage gateway routes"</p>

                <button
                    class="btn btn-primary"
                    on:click=on_add_click
                    disabled=move || show_form.get()
                >
                    "➕ Add Route"
                </button>
            </div>

            // Show error/success messages
            {move || form_error.get().map(|err| view! {
                <div class="alert alert-error">
                    <span class="alert-icon">"⚠️"</span>
                    <span>{err}</span>
                </div>
            })}

            {move || form_success.get().map(|msg| view! {
                <div class="alert alert-success">
                    <span class="alert-icon">"✅"</span>
                    <span>{msg}</span>
                </div>
            })}

            // Route form (shown when adding/editing)
            {move || show_form.get().then(|| view! {
                <RouteForm
                    route=editing_route.get()
                    on_submit=on_submit
                    on_cancel=on_cancel
                />
            })}

            // Routes list
            <Suspense fallback=move || view! { <LoadingSpinner /> }>
                {move || Suspend::new(async move {
                    match routes_resource.await {
                        Ok(routes) => view! {
                            <RoutesList
                                routes=routes
                                on_edit=on_edit
                                on_delete=on_delete
                            />
                        }.into_any(),
                        Err(e) => view! {
                            <div class="error-state">
                                <div class="error-icon">"❌"</div>
                                <h3>"Failed to load routes"</h3>
                                <p>{format!("Error: {}", e)}</p>
                            </div>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq)]
enum RouteFormTab {
    Basic,
    Backends,
    Retry,
    AiPolicy,
}

/// Form component for creating/editing routes.
#[component]
fn RouteForm(
    route: Option<Router>,
    on_submit: impl Fn(Router) + 'static + Copy,
    on_cancel: impl Fn(()) + 'static + Copy,
) -> impl IntoView {
    let is_editing = route.is_some();
    let mut initial_route = route.unwrap_or_default();

    // Normalize legacy host/port to backends if they exist but backends doesn't
    if initial_route.backends.is_none() {
        if let (Some(h), Some(p)) = (&initial_route.host, initial_route.port) {
            initial_route.backends = Some(vec![Backend {
                host: h.clone(),
                port: p,
                weight: 1,
                health_check_path: None,
            }]);
            initial_route.host = None;
            initial_route.port = None;
        }
    }

    let (draft, set_draft) = signal(initial_route);
    let (active_tab, set_active_tab) = signal(RouteFormTab::Basic);
    let (validation_error, set_validation_error) = signal::<Option<String>>(None);

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_validation_error.set(None);

        let mut final_route = draft.get();
        // Clear legacy host/port since we migrate them to backends internally
        final_route.host = None;
        final_route.port = None;

        match final_route.validate() {
            Ok(_) => on_submit(final_route),
            Err(e) => set_validation_error.set(Some(e)),
        }
    };

    let available_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

    view! {
        <div class="route-form-container">
            <div class="form-header">
                <h2>{if is_editing { "Edit Route" } else { "Create New Route" }}</h2>
            </div>

            {move || validation_error.get().map(|err| view! {
                <div class="alert alert-error">
                    <span class="alert-icon">"⚠️"</span>
                    <span>{err}</span>
                </div>
            })}

            <div class="tabs-nav" style="margin-bottom: 20px;">
                <button
                    class=move || if active_tab.get() == RouteFormTab::Basic { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_tab.set(RouteFormTab::Basic)
                    type="button"
                >
                    "📄 Basic"
                </button>
                <button
                    class=move || if active_tab.get() == RouteFormTab::Backends { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_tab.set(RouteFormTab::Backends)
                    type="button"
                >
                    "⚖️ Backends"
                </button>
                <button
                    class=move || if active_tab.get() == RouteFormTab::Retry { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_tab.set(RouteFormTab::Retry)
                    type="button"
                >
                    "🔄 Retry"
                </button>
                <button
                    class=move || if active_tab.get() == RouteFormTab::AiPolicy { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_tab.set(RouteFormTab::AiPolicy)
                    type="button"
                >
                    "🤖 AI Policy"
                </button>
            </div>

            <form on:submit=handle_submit class="route-form">
                <div class="tab-content tab-content-wrapper">
                    // --- BASIC TAB ---
                    <div style=move || if active_tab.get() == RouteFormTab::Basic { "display: block;" } else { "display: none;" }>
                        <div class="form-grid">
                            <div class="form-group">
                                <label for="external-path">"External Path"</label>
                                <input
                                    type="text"
                                    id="external-path"
                                    class="form-input"
                                    placeholder="/api/users/{id}"
                                    prop:value=move || draft.get().external_path
                                    on:input=move |ev| set_draft.update(|d| d.external_path = event_target_value(&ev))
                                    required
                                />
                                <small class="form-help">"Path that clients will use (supports {param} placeholders)"</small>
                            </div>
                            <div class="form-group">
                                <label for="internal-path">"Internal Path"</label>
                                <input
                                    type="text"
                                    id="internal-path"
                                    class="form-input"
                                    placeholder="/v1/users/{id}"
                                    prop:value=move || draft.get().internal_path
                                    on:input=move |ev| set_draft.update(|d| d.internal_path = event_target_value(&ev))
                                    required
                                />
                                <small class="form-help">"Path to forward to backend service"</small>
                            </div>
                            <div class="form-group">
                                <label>"Protocol"</label>
                                <select
                                    class="form-select"
                                    on:change=move |ev| {
                                        let val = event_target_value(&ev);
                                        set_draft.update(|d| {
                                            d.protocol = match val.as_str() {
                                                "websocket" => Protocol::WebSocket,
                                                "ftp" => Protocol::Ftp,
                                                "dns" => Protocol::Dns,
                                                _ => Protocol::Http,
                                            };
                                        });
                                    }
                                >
                                    <option value="http" selected=move || draft.get().protocol == Protocol::Http>"HTTP"</option>
                                    <option value="websocket" selected=move || draft.get().protocol == Protocol::WebSocket>"WebSocket"</option>
                                </select>
                            </div>
                        </div>

                        <div class="form-group">
                            <label>"HTTP Methods"</label>
                            <div class="methods-grid">
                                {available_methods.iter().map(|&method| {
                                    let is_selected = move || draft.get().methods.contains(&method.to_string());
                                    view! {
                                        <button
                                            type="button"
                                            class=move || if is_selected() { "method-btn method-btn-active" } else { "method-btn" }
                                            on:click=move |_| {
                                                set_draft.update(|d| {
                                                    if d.methods.contains(&method.to_string()) {
                                                        d.methods.retain(|x| x != method);
                                                    } else {
                                                        d.methods.push(method.to_string());
                                                    }
                                                });
                                            }
                                        >
                                            {method}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>

                        <div class="form-group">
                            <label class="checkbox-label">
                                <input
                                    type="checkbox"
                                    prop:checked=move || draft.get().auth_required
                                    on:change=move |ev| set_draft.update(|d| d.auth_required = event_target_checked(&ev))
                                />
                                <span>"Require JWT Authentication"</span>
                            </label>
                        </div>
                    </div>

                    // --- BACKENDS TAB ---
                    <div style=move || if active_tab.get() == RouteFormTab::Backends { "display: block;" } else { "display: none;" }>
                        <div class="form-group" style="margin-bottom: 20px;">
                            <label>"Load Balancing Strategy"</label>
                            <select
                                class="form-select"
                                on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                    let strat = match val.as_str() {
                                        "least_connections" => LoadBalancingStrategy::LeastConnections,
                                        "random" => LoadBalancingStrategy::Random,
                                        "weighted" => LoadBalancingStrategy::Weighted,
                                        "ip_hash" => LoadBalancingStrategy::IpHash,
                                        _ => LoadBalancingStrategy::RoundRobin,
                                    };
                                    set_draft.update(|d| d.load_balancing_strategy = strat);
                                }
                            >
                                <option value="round_robin" selected=move || draft.get().load_balancing_strategy == LoadBalancingStrategy::RoundRobin>"Round Robin"</option>
                                <option value="least_connections" selected=move || draft.get().load_balancing_strategy == LoadBalancingStrategy::LeastConnections>"Least Connections"</option>
                                <option value="random" selected=move || draft.get().load_balancing_strategy == LoadBalancingStrategy::Random>"Random"</option>
                                <option value="weighted" selected=move || draft.get().load_balancing_strategy == LoadBalancingStrategy::Weighted>"Weighted"</option>
                                <option value="ip_hash" selected=move || draft.get().load_balancing_strategy == LoadBalancingStrategy::IpHash>"IP Hash"</option>
                            </select>
                        </div>

                        <div class="form-group">
                            <div class="tabs-nav">
                                <label style="margin: 0; font-weight: bold;">"Backend Targets"</label>
                                <button type="button" class="btn btn-sm btn-secondary" on:click=move |_| {
                                    set_draft.update(|d| {
                                        let mut backs = d.backends.clone().unwrap_or_default();
                                        backs.push(Backend {
                                            host: "http://localhost".to_string(),
                                            port: 8080,
                                            weight: 1,
                                            health_check_path: None,
                                        });
                                        d.backends = Some(backs);
                                    });
                                }>
                                    "➕ Add Backend"
                                </button>
                            </div>

                            <div class="backends-list" style="display: flex; flex-direction: column; gap: 10px;">
                                {move || {
                                    let backends = draft.get().backends.unwrap_or_default();
                                    if backends.is_empty() {
                                        view! { <div class="empty-state" style="padding: 20px; text-align: center;">"No backends configured. Requests will fail."</div> }.into_any()
                                    } else {
                                        backends.into_iter().enumerate().map(|(idx, backend)| {
                                            view! {
                                                <div class="backend-item backend-item-card">
                                                    <div style="flex: 2;">
                                                        <label class="form-label">"Host"</label>
                                                        <input type="text" class="form-input" prop:value=backend.host.clone() on:input=move |ev| {
                                                            set_draft.update(|d| if let Some(ref mut b) = d.backends { b[idx].host = event_target_value(&ev) });
                                                        } />
                                                    </div>
                                                    <div style="flex: 1;">
                                                        <label class="form-label">"Port"</label>
                                                        <input type="number" class="form-input" min="1" max="65535" prop:value=backend.port on:input=move |ev| {
                                                            let p = event_target_value(&ev).parse::<u16>().unwrap_or(0);
                                                            set_draft.update(|d| if let Some(ref mut b) = d.backends { b[idx].port = p });
                                                        } />
                                                    </div>
                                                    <div style="flex: 1;">
                                                        <label class="form-label">"Weight"</label>
                                                        <input type="number" class="form-input" min="1" prop:value=backend.weight on:input=move |ev| {
                                                            let w = event_target_value(&ev).parse::<u32>().unwrap_or(0);
                                                            set_draft.update(|d| if let Some(ref mut b) = d.backends { b[idx].weight = w });
                                                        } />
                                                    </div>
                                                    <button type="button" class="btn-icon btn-icon-delete backend-delete-btn" on:click=move |_| {
                                                        let host = backend.host.clone();
                                                        let port = backend.port;
                                                        set_draft.update(|d| {
                                                            if let Some(ref mut b) = d.backends {
                                                                if let Some(pos) = b.iter().position(|be| be.host == host && be.port == port) {
                                                                    b.remove(pos);
                                                                }
                                                            }
                                                        });
                                                    }>
                                                        "🗑️"
                                                    </button>
                                                </div>
                                            }
                                        }).collect_view().into_any()
                                    }
                                }}
                            </div>
                        </div>
                    </div>

                    // --- RETRY TAB ---
                    <div style=move || if active_tab.get() == RouteFormTab::Retry { "display: block;" } else { "display: none;" }>
                        <div class="form-group">
                            <label class="checkbox-label" style="font-weight: bold; margin-bottom: 20px;">
                                <input
                                    type="checkbox"
                                    prop:checked=move || draft.get().retry.is_some()
                                    on:change=move |ev| {
                                        let holds = event_target_checked(&ev);
                                        set_draft.update(|d| {
                                            if holds {
                                                d.retry = Some(RetryConfig::default());
                                            } else {
                                                d.retry = None;
                                            }
                                        });
                                    }
                                />
                                <span>"Enable Automatic Retries"</span>
                            </label>

                            {move || {
                                if let Some(retry) = draft.get().retry {
                                    view! {
                                        <div class="form-grid" style="margin-top: 20px; padding: 15px; border: 1px dashed var(--border-color); border-radius: 6px;">
                                            <div class="form-group">
                                                <label>"Max Retries"</label>
                                                <input type="number" class="form-input" min="1" max="10" prop:value=retry.max_retries on:input=move |ev| {
                                                    if let Ok(v) = event_target_value(&ev).parse::<u32>() {
                                                        set_draft.update(|d| if let Some(ref mut r) = d.retry { r.max_retries = v });
                                                    }
                                                } />
                                            </div>
                                            <div class="form-group">
                                                <label>"Initial Backoff (ms)"</label>
                                                <input type="number" class="form-input" min="1" prop:value=retry.initial_backoff_ms on:input=move |ev| {
                                                    if let Ok(v) = event_target_value(&ev).parse::<u64>() {
                                                        set_draft.update(|d| if let Some(ref mut r) = d.retry { r.initial_backoff_ms = v });
                                                    }
                                                } />
                                            </div>
                                            <div class="form-group">
                                                <label>"Max Backoff (ms)"</label>
                                                <input type="number" class="form-input" min="1" prop:value=retry.max_backoff_ms on:input=move |ev| {
                                                    if let Ok(v) = event_target_value(&ev).parse::<u64>() {
                                                        set_draft.update(|d| if let Some(ref mut r) = d.retry { r.max_backoff_ms = v });
                                                    }
                                                } />
                                            </div>
                                            <div class="form-group">
                                                <label>"Backoff Multiplier"</label>
                                                <input type="number" step="0.1" class="form-input" min="1.0" prop:value=retry.backoff_multiplier on:input=move |ev| {
                                                    if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                                        set_draft.update(|d| if let Some(ref mut r) = d.retry { r.backoff_multiplier = v });
                                                    }
                                                } />
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div class="form-help">"Retry logic is disabled. Failed requests will return immediately to the client."</div> }.into_any()
                                }
                            }}
                        </div>
                    </div>

                    // --- AI POLICY TAB ---
                    <div style=move || if active_tab.get() == RouteFormTab::AiPolicy { "display: block;" } else { "display: none;" }>
                        <div class="form-group">
                            <label class="checkbox-label" style="font-weight: bold; margin-bottom: 20px;">
                                <input
                                    type="checkbox"
                                    prop:checked=move || draft.get().ai_policy.is_some()
                                    on:change=move |ev| {
                                        let holds = event_target_checked(&ev);
                                        set_draft.update(|d| {
                                            if holds {
                                                d.ai_policy = Some(AiPolicy {
                                                    enabled: true,
                                                    strategy: AiRoutingStrategy::ContentAnalysis { model: None },
                                                    provider: None,
                                                    fallback_backend_index: None,
                                                });
                                            } else {
                                                d.ai_policy = None;
                                            }
                                        });
                                    }
                                />
                                <span>"Enable AI-Assisted Smart Routing"</span>
                            </label>

                            {move || {
                                if draft.get().ai_policy.is_some() {
                                    view! {
                                        <div class="form-grid" style="margin-top: 20px; padding: 15px; border: 1px solid var(--accent-light); border-radius: 6px; background-color: rgba(var(--accent-rgb), 0.05);">
                                            <div class="form-group">
                                                <label>"Evaluation Strategy"</label>
                                                <select
                                                    class="form-select"
                                                    on:change=move |ev| {
                                                        let val = event_target_value(&ev);
                                                        set_draft.update(|d| {
                                                            if let Some(ref mut a) = d.ai_policy {
                                                                a.strategy = match val.as_str() {
                                                                    "latency" => AiRoutingStrategy::LatencyPrediction,
                                                                    "anomaly" => AiRoutingStrategy::AnomalyDetection,
                                                                    _ => AiRoutingStrategy::ContentAnalysis { model: None },
                                                                };
                                                            }
                                                        });
                                                    }
                                                >
                                                    <option value="content" selected=move || matches!(draft.get().ai_policy.map(|a| a.strategy), Some(AiRoutingStrategy::ContentAnalysis{..}))>"Content Analysis (Routing based on payload)"</option>
                                                    <option value="latency" selected=move || matches!(draft.get().ai_policy.map(|a| a.strategy), Some(AiRoutingStrategy::LatencyPrediction))>"Latency Prediction (Route to fastest backend)"</option>
                                                    <option value="anomaly" selected=move || matches!(draft.get().ai_policy.map(|a| a.strategy), Some(AiRoutingStrategy::AnomalyDetection))>"Anomaly Detection (Block malicious payloads)"</option>
                                                </select>
                                            </div>
                                            <div class="form-group">
                                                <label>"Override AI Provider"</label>
                                                <input type="text" class="form-input" placeholder="e.g. openai (optional, uses default if empty)" prop:value=move || draft.get().ai_policy.and_then(|a| a.provider).unwrap_or_default() on:input=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    set_draft.update(|d| if let Some(ref mut a) = d.ai_policy {
                                                        a.provider = if val.is_empty() { None } else { Some(val) };
                                                    });
                                                } />
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div class="form-help">"AI features are disabled for this route."</div> }.into_any()
                                }
                            }}
                        </div>
                    </div>
                </div>

                <div class="form-actions">
                    <button type="submit" class="btn btn-primary">
                        {if is_editing { "💾 Update Route" } else { "💾 Create Route" }}
                    </button>
                    <button type="button" class="btn btn-secondary" on:click=move |_| on_cancel(())>
                        "❌ Cancel"
                    </button>
                </div>
            </form>
        </div>
    }
}

/// Component for displaying routes list.
#[component]
fn RoutesList(
    routes: Vec<Router>,
    on_edit: impl Fn(Router) + 'static + Copy,
    on_delete: impl Fn(String) + 'static + Copy,
) -> impl IntoView {
    if routes.is_empty() {
        return view! {
            <div class="empty-state">
                <div class="empty-icon">"📭"</div>
                <h3>"No routes configured"</h3>
                <p>"Click 'Add Route' above to create your first route"</p>
            </div>
        }
        .into_any();
    }

    view! {
        <div class="routes-table-container">
            <table class="routes-table">
                <thead>
                    <tr>
                        <th>"Route Path"</th>
                        <th>"Backends"</th>
                        <th>"Protocol"</th>
                        <th>"Methods"</th>
                        <th>"Features"</th>
                        <th>"Actions"</th>
                    </tr>
                </thead>
                <tbody>
                    {routes.into_iter().map(|route| {
                        let external_path = route.external_path.clone();
                        // Compute backends count
                        let backend_count = if let Some(ref backends) = route.backends {
                            backends.len()
                        } else if route.host.is_some() {
                            1
                        } else {
                            0
                        };

                        let route_for_edit = route.clone();
                        let route_path_for_delete = external_path.clone();
                        view! {
                            <tr>
                                <td>
                                    <div style="display: flex; flex-direction: column; gap: 4px;">
                                        <strong>{route.external_path.clone()}</strong>
                                        <small style="color: var(--text-muted); font-family: monospace;">"→ "{route.internal_path.clone()}</small>
                                    </div>
                                </td>
                                <td>
                                    <span class="backend-badge">{format!("{} target(s)", backend_count)}</span>
                                </td>
                                <td>
                                    <span class="method-tag" style="background: var(--bg-hover); color: var(--text-color);">
                                        {match route.protocol {
                                            Protocol::Http => "HTTP",
                                            Protocol::WebSocket => "WS",
                                            Protocol::Ftp => "FTP",
                                            Protocol::Dns => "DNS",
                                        }}
                                    </span>
                                </td>
                                <td>
                                    <div class="methods-list">
                                        {route.methods.iter().map(|m| view! {
                                            <span class="method-tag">{m.clone()}</span>
                                        }).collect_view()}
                                    </div>
                                </td>
                                <td>
                                    <div style="display: flex; flex-direction: column; gap: 4px; font-size: 0.8em;">
                                        {route.auth_required.then(|| view! { <span style="color: var(--success-color);">"🔒 Auth"</span> })}
                                        {route.retry.is_some().then(|| view! { <span style="color: var(--primary-color);">"🔄 Retry"</span> })}
                                        {route.ai_policy.is_some().then(|| view! { <span style="color: var(--accent-color);">"🤖 AI Opt"</span> })}
                                    </div>
                                </td>
                                <td class="actions-cell">
                                    <button
                                        class="btn-icon btn-icon-edit"
                                        on:click=move |_| on_edit(route_for_edit.clone())
                                        title="Edit route"
                                    >
                                        "✏️"
                                    </button>
                                    <button
                                        class="btn-icon btn-icon-delete"
                                        on:click=move |_| {
                                            if web_sys::window()
                                                .and_then(|w| w.confirm_with_message(&format!("Delete route '{}'?", route_path_for_delete)).ok())
                                                .unwrap_or(false)
                                            {
                                                on_delete(route_path_for_delete.clone());
                                            }
                                        }
                                        title="Delete route"
                                    >
                                        "🗑️"
                                    </button>
                                </td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }.into_any()
}
