use crate::models::AiSettings;
use leptos::prelude::*;

#[component]
pub fn AiConfigForm<F>(ai_settings: Option<AiSettings>, on_save: F) -> impl IntoView
where
    F: Fn(AiSettings) + 'static + Clone,
{
    let initial = ai_settings.unwrap_or_default();
    let (provider, set_provider) = signal(initial.provider.clone());
    let (model, set_model) = signal(initial.model.clone());
    let (api_key, set_api_key) = signal(initial.api_key.clone().unwrap_or_default());
    let (validation_error, set_validation_error) = signal(None::<String>);

    let handle_save = move |_| {
        set_validation_error.set(None);

        if provider.get().is_empty() {
            set_validation_error.set(Some("Provider is required".to_string()));
            return;
        }

        if model.get().is_empty() {
            set_validation_error.set(Some("Model is required".to_string()));
            return;
        }

        let config = AiSettings {
            provider: provider.get(),
            model: model.get(),
            api_key: if api_key.get().is_empty() {
                None
            } else {
                Some(api_key.get())
            },
        };

        on_save(config);
    };

    view! {
        <div class="config-form ai-config">
            <h2>"AI Configuration"</h2>
            <p class="form-description">
                "Configure AI provider settings for routing and semantic operations."
            </p>

            {move || validation_error.get().map(|err| view! {
                <div class="alert alert-error">
                    <span class="alert-icon">"⚠️"</span>
                    <span class="alert-message">{err}</span>
                </div>
            })}

            <div class="form-row">
                <div class="form-group">
                    <label class="form-label required">"Provider"</label>
                    <select
                        class="form-select"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            set_provider.set(value);
                        }
                    >
                        <option value="openai" selected=move || provider.get() == "openai">"OpenAI"</option>
                        <option value="claude" selected=move || provider.get() == "claude">"Claude (Anthropic)"</option>
                        <option value="cohere" selected=move || provider.get() == "cohere">"Cohere"</option>
                        <option value="gemini" selected=move || provider.get() == "gemini">"Gemini (Google)"</option>
                        <option value="xai" selected=move || provider.get() == "xai">"xAI (Grok)"</option>
                    </select>
                </div>

                <div class="form-group">
                    <label class="form-label required">"Model"</label>
                    <input
                        type="text"
                        class="form-input"
                        placeholder="e.g., gpt-4, claude-3-opus-20240229"
                        prop:value=move || model.get()
                        on:input=move |ev| set_model.set(event_target_value(&ev))
                    />
                </div>
            </div>

            <div class="form-group">
                <label class="form-label">
                    "API Key"
                    <span class="label-hint">"(optional, overrides environment variables)"</span>
                </label>
                <input
                    type="password"
                    class="form-input"
                    placeholder="Enter provider API key"
                    prop:value=move || api_key.get()
                    on:input=move |ev| set_api_key.set(event_target_value(&ev))
                />
            </div>

            <div class="form-actions">
                <button class="btn btn-primary" on:click=handle_save>
                    "💾 Save AI Configuration"
                </button>
            </div>
        </div>
    }
}
