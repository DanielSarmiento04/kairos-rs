use leptos::prelude::*;
use crate::models::metrics::SystemMetrics;

#[component]
pub fn RealTimeMetrics() -> impl IntoView {
    let (metrics, set_metrics) = signal(SystemMetrics::default());
    let (status, set_status) = signal("Disconnected");

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            use web_sys::{WebSocket, MessageEvent};
            use wasm_bindgen::prelude::*;
            use wasm_bindgen::JsCast;
            use send_wrapper::SendWrapper;

            let window = web_sys::window().unwrap();
            let location = window.location();
            let protocol = if location.protocol().unwrap() == "https:" { "wss:" } else { "ws:" };
            let host = location.host().unwrap();
            let ws_url = format!("{}//{}/ws/admin/metrics", protocol, host);

            if let Ok(ws) = WebSocket::new(&ws_url) {
                set_status.set("Connecting...");

                let onopen_callback = Closure::<dyn FnMut()>::new(move || {
                    set_status.set("Connected");
                });
                ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
                onopen_callback.forget();

                let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                    if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                        let txt: String = txt.into();
                        if let Ok(m) = serde_json::from_str::<SystemMetrics>(&txt) {
                            set_metrics.set(m);
                        }
                    }
                });
                ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                onmessage_callback.forget();

                let onclose_callback = Closure::<dyn FnMut()>::new(move || {
                    set_status.set("Disconnected");
                });
                ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
                onclose_callback.forget();
                
                let ws_wrapper = SendWrapper::new(ws);
                on_cleanup(move || {
                    let _ = ws_wrapper.take().close();
                });
            } else {
                set_status.set("Connection Failed");
            }
        }
    });

    view! {
        <div class="real-time-metrics-container">
            <div class="metrics-header">
                <h3>"Real-time System Metrics"</h3>
                <span class=move || format!("status-badge {}", status.get().to_lowercase())>
                    {status}
                </span>
            </div>
            <div class="metrics-grid">
                <div class="metric-card">
                    <span class="metric-label">"CPU Usage"</span>
                    <span class="metric-value">{move || format!("{:.1}%", metrics.get().cpu_usage)}</span>
                </div>
                <div class="metric-card">
                    <span class="metric-label">"Memory"</span>
                    <span class="metric-value">{move || format!("{:.0} MB", metrics.get().memory_usage as f64 / 1024.0 / 1024.0)}</span>
                </div>
                <div class="metric-card">
                    <span class="metric-label">"Active Connections"</span>
                    <span class="metric-value">{move || metrics.get().active_connections}</span>
                </div>
                <div class="metric-card">
                    <span class="metric-label">"Uptime"</span>
                    <span class="metric-value">{move || format_duration(metrics.get().uptime)}</span>
                </div>
            </div>
        </div>
    }
}

fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else {
        format!("{}m {}s", minutes, secs)
    }
}
