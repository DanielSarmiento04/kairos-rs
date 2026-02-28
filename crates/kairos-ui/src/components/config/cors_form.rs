use crate::models::CorsConfig;
use leptos::prelude::*;

#[component]
pub fn CorsConfigForm<F>(cors_config: Option<CorsConfig>, on_save: F) -> impl IntoView
where
    F: Fn(CorsConfig) + 'static + Clone,
{
    let initial = cors_config.unwrap_or_default();
    let (enabled, set_enabled) = signal(initial.enabled);
    let (allowed_origins, set_allowed_origins) = signal(initial.allowed_origins.join("\n"));
    let (allowed_methods, set_allowed_methods) = signal(initial.allowed_methods.join(", "));
    let (allowed_headers, set_allowed_headers) = signal(initial.allowed_headers.join(", "));
    let (allow_credentials, set_allow_credentials) = signal(initial.allow_credentials);
    let (max_age, set_max_age) = signal(
        initial
            .max_age_secs
            .map(|v| v.to_string())
            .unwrap_or_default(),
    );

    let handle_save = move |_| {
        let origins: Vec<String> = allowed_origins
            .get()
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let methods: Vec<String> = allowed_methods
            .get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let headers: Vec<String> = allowed_headers
            .get()
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
