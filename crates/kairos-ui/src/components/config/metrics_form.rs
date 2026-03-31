use crate::models::MetricsConfig;
use leptos::prelude::*;

#[component]
pub fn MetricsConfigForm<F>(metrics_config: Option<MetricsConfig>, on_save: F) -> impl IntoView
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
