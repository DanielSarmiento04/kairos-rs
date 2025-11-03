//! Routes management page for viewing and editing gateway routes.
//!
//! This page provides a comprehensive CRUD interface for managing gateway routes,
//! including creating, updating, deleting, and testing route configurations.

use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::components::*;
use crate::models::router::*;
use crate::server_functions::api::*;

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
                    set_form_success.set(Some(format!("Route '{}' deleted successfully!", external_path)));
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
                    set_form_success.set(Some(format!("Route '{}' {} successfully!", route.external_path, action)));
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

/// Form component for creating/editing routes.
#[component]
fn RouteForm(
    route: Option<Router>,
    on_submit: impl Fn(Router) + 'static + Copy,
    on_cancel: impl Fn(()) + 'static + Copy,
) -> impl IntoView {
    let is_editing = route.is_some();
    let initial_route = route.unwrap_or_default();
    
    // Form state
    let (external_path, set_external_path) = signal(initial_route.external_path.clone());
    let (internal_path, set_internal_path) = signal(initial_route.internal_path.clone());
    let (backend_host, set_backend_host) = signal(
        initial_route.host.clone().or_else(|| 
            initial_route.backends.as_ref()
                .and_then(|b| b.first().map(|backend| backend.host.clone()))
        ).unwrap_or_else(|| "http://localhost".to_string())
    );
    let (backend_port, set_backend_port) = signal(
        initial_route.port.unwrap_or_else(|| 
            initial_route.backends.as_ref()
                .and_then(|b| b.first().map(|backend| backend.port))
                .unwrap_or(8080)
        )
    );
    let (methods, set_methods) = signal(initial_route.methods.clone());
    let (auth_required, set_auth_required) = signal(initial_route.auth_required);
    let (validation_error, set_validation_error) = signal::<Option<String>>(None);
    
    // Toggle method selection
    let toggle_method = move |method: &'static str| {
        set_methods.update(|m| {
            if m.contains(&method.to_string()) {
                m.retain(|x| x != method);
            } else {
                m.push(method.to_string());
            }
        });
    };
    
    // Handle form submission
    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_validation_error.set(None);
        
        let route = Router {
            host: Some(backend_host.get()),
            port: Some(backend_port.get()),
            backends: None,
            protocol: Protocol::default(),
            load_balancing_strategy: LoadBalancingStrategy::default(),
            external_path: external_path.get(),
            internal_path: internal_path.get(),
            methods: methods.get(),
            auth_required: auth_required.get(),
            retry: None,
        };
        
        match route.validate() {
            Ok(_) => on_submit(route),
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
                    <span>{err}</span>
                </div>
            })}
            
            <form on:submit=handle_submit class="route-form">
                <div class="form-grid">
                    <div class="form-group">
                        <label for="external-path">"External Path"</label>
                        <input 
                            type="text"
                            id="external-path"
                            class="form-input"
                        placeholder="/api/users/{id}"
                        prop:value=move || external_path.get()
                        on:input=move |ev| set_external_path.set(event_target_value(&ev))
                        required
                    />
                    <small class="form-help">"Path that clients will use (supports " {r#"{param}"#} " placeholders)"</small>
                </div>                    <div class="form-group">
                        <label for="internal-path">"Internal Path"</label>
                        <input 
                            type="text"
                            id="internal-path"
                            class="form-input"
                            placeholder="/v1/users/{id}"
                            prop:value=move || internal_path.get()
                            on:input=move |ev| set_internal_path.set(event_target_value(&ev))
                            required
                        />
                        <small class="form-help">"Path to forward to backend service"</small>
                    </div>
                    
                    <div class="form-group">
                        <label for="backend-host">"Backend Host"</label>
                        <input 
                            type="text"
                            id="backend-host"
                            class="form-input"
                            placeholder="http://backend-service"
                            prop:value=move || backend_host.get()
                            on:input=move |ev| set_backend_host.set(event_target_value(&ev))
                            required
                        />
                        <small class="form-help">"Backend server URL (include http:// or https://)"</small>
                    </div>
                    
                    <div class="form-group">
                        <label for="backend-port">"Backend Port"</label>
                        <input 
                            type="number"
                            id="backend-port"
                            class="form-input"
                            min="1"
                            max="65535"
                            prop:value=move || backend_port.get()
                            on:input=move |ev| {
                                if let Ok(port) = event_target_value(&ev).parse::<u16>() {
                                    set_backend_port.set(port);
                                }
                            }
                            required
                        />
                    </div>
                </div>
                
                <div class="form-group">
                    <label>"HTTP Methods"</label>
                    <div class="methods-grid">
                        {available_methods.iter().map(|&method| {
                            let is_selected = move || methods.get().contains(&method.to_string());
                            view! {
                                <button
                                    type="button"
                                    class=move || if is_selected() { "method-btn method-btn-active" } else { "method-btn" }
                                    on:click=move |_| toggle_method(method)
                                >
                                    {method}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                    <small class="form-help">"Select one or more HTTP methods"</small>
                </div>
                
                <div class="form-group">
                    <label class="checkbox-label">
                        <input 
                            type="checkbox"
                            prop:checked=move || auth_required.get()
                            on:change=move |ev| set_auth_required.set(event_target_checked(&ev))
                        />
                        <span>"Require JWT Authentication"</span>
                    </label>
                </div>
                
                <div class="form-actions">
                    <button type="submit" class="btn btn-primary">
                        {if is_editing { "Update Route" } else { "Create Route" }}
                    </button>
                    <button type="button" class="btn btn-secondary" on:click=move |_| on_cancel(())>
                        "Cancel"
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
        }.into_any();
    }
    
    view! {
        <div class="routes-table-container">
            <table class="routes-table">
                <thead>
                    <tr>
                        <th>"External Path"</th>
                        <th>"Internal Path"</th>
                        <th>"Backend"</th>
                        <th>"Methods"</th>
                        <th>"Auth"</th>
                        <th>"Actions"</th>
                    </tr>
                </thead>
                <tbody>
                    {routes.into_iter().map(|route| {
                        let external_path = route.external_path.clone();
                        let backend = if let Some(ref backends) = route.backends {
                            backends.first()
                                .map(|b| format!("{}:{}", b.host, b.port))
                                .unwrap_or_else(|| "N/A".to_string())
                        } else if let (Some(host), Some(port)) = (&route.host, &route.port) {
                            format!("{}:{}", host, port)
                        } else {
                            "N/A".to_string()
                        };
                        
                        let route_for_edit = route.clone();
                        let route_path_for_delete = external_path.clone();
                        
                        view! {
                            <tr>
                                <td><code>{route.external_path.clone()}</code></td>
                                <td><code>{route.internal_path.clone()}</code></td>
                                <td><span class="backend-badge">{backend}</span></td>
                                <td>
                                    <div class="methods-list">
                                        {route.methods.iter().map(|m| view! {
                                            <span class="method-tag">{m.clone()}</span>
                                        }).collect_view()}
                                    </div>
                                </td>
                                <td>
                                    <StatusBadge 
                                        variant=if route.auth_required { StatusVariant::Success } else { StatusVariant::Info }
                                        text=if route.auth_required { "Required".to_string() } else { "None".to_string() }
                                    />
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
