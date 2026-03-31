use crate::models::JwtSettings;
use leptos::prelude::*;

#[component]
pub fn JwtConfigForm<F>(jwt_settings: Option<JwtSettings>, on_save: F) -> impl IntoView
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

        let claims: Vec<String> = required_claims
            .get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let config = JwtSettings {
            secret: secret.get(),
            issuer: if issuer.get().is_empty() {
                None
            } else {
                Some(issuer.get())
            },
            audience: if audience.get().is_empty() {
                None
            } else {
                Some(audience.get())
            },
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
