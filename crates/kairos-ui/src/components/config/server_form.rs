use crate::models::ServerConfig;
use leptos::prelude::*;

#[component]
pub fn ServerConfigForm<F>(server_config: Option<ServerConfig>, on_save: F) -> impl IntoView
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
                set_validation_error.set(Some(
                    "Port must be a valid number between 1-65535".to_string(),
                ));
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
