use crate::models::{LimitStrategy, RateLimitConfig, WindowType};
use leptos::prelude::*;

#[component]
pub fn RateLimitConfigForm<F>(rate_limit: Option<RateLimitConfig>, on_save: F) -> impl IntoView
where
    F: Fn(RateLimitConfig) + 'static + Clone,
{
    let initial = rate_limit.unwrap_or_default();
    let (strategy, set_strategy) = signal(initial.strategy);
    let (requests_per_window, set_requests_per_window) =
        signal(initial.requests_per_window.to_string());
    let (window_duration, set_window_duration) = signal(initial.window_duration.to_string());
    let (burst_allowance, set_burst_allowance) = signal(initial.burst_allowance.to_string());
    let (window_type, set_window_type) = signal(initial.window_type);
    let (enable_redis, set_enable_redis) = signal(initial.enable_redis);
    let (redis_prefix, set_redis_prefix) = signal(initial.redis_key_prefix.clone());
    let (validation_error, set_validation_error) = signal(None::<String>);

    let handle_save = move |_| {
        set_validation_error.set(None);

        let requests: u64 = match requests_per_window.get().parse() {
            Ok(v) if v > 0 => v,
            _ => {
                set_validation_error.set(Some(
                    "Requests per window must be a positive number".to_string(),
                ));
                return;
            }
        };

        let duration: u64 = match window_duration.get().parse() {
            Ok(v) if v > 0 => v,
            _ => {
                set_validation_error.set(Some(
                    "Window duration must be a positive number".to_string(),
                ));
                return;
            }
        };

        let burst: u64 = match burst_allowance.get().parse() {
            Ok(v) => v,
            _ => {
                set_validation_error
                    .set(Some("Burst allowance must be a valid number".to_string()));
                return;
            }
        };

        let config = RateLimitConfig {
            strategy: strategy.get(),
            requests_per_window: requests,
            window_duration: duration,
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
