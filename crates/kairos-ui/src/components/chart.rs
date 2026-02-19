use leptos::prelude::*;
use send_wrapper::SendWrapper;
use serde_json::Value;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
extern "C" {
    type Chart;

    #[wasm_bindgen(constructor)]
    fn new(ctx: &HtmlCanvasElement, config: JsValue) -> Chart;

    #[wasm_bindgen(method)]
    fn destroy(this: &Chart);

    #[wasm_bindgen(method)]
    fn update(this: &Chart);
}

#[component]
pub fn Chart(
    #[prop(into)] id: String,
    #[prop(into)] kind: String, // "line", "bar", etc.
    #[prop(into)] data: Value,
    #[prop(into, optional)] options: Option<Value>,
) -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    // Use SendWrapper to satisfy StoredValue's Send requirement
    // Chart is a wasm_bindgen type which is !Send
    let chart_instance = StoredValue::new(None::<SendWrapper<Chart>>);

    // Effect to initialize/update chart when data changes
    Effect::new(move |_| {
        let data = data.clone();
        let options = options.clone();
        let kind = kind.clone();

        #[cfg(feature = "hydrate")]
        request_animation_frame(move || {
            if let Some(canvas) = canvas_ref.get() {
                // Destroy existing chart if it exists
                chart_instance.update_value(|chart_opt| {
                    if let Some(wrapper) = chart_opt {
                        wrapper.destroy();
                    }
                });

                // Prepare config
                let config = serde_json::json!({
                    "type": kind,
                    "data": data,
                    "options": options.unwrap_or(serde_json::json!({
                        "responsive": true,
                        "maintainAspectRatio": false
                    }))
                });

                // Create new chart
                if let Ok(config_js) = serde_wasm_bindgen::to_value(&config) {
                    let chart = Chart::new(&canvas, config_js);
                    chart_instance.set_value(Some(SendWrapper::new(chart)));
                }
            }
        });
    });

    on_cleanup(move || {
        chart_instance.update_value(|chart_opt| {
            if let Some(wrapper) = chart_opt {
                wrapper.destroy();
            }
        });
    });

    view! {
        <div class="chart-wrapper" style="position: relative; height: 100%; width: 100%;">
            <canvas node_ref=canvas_ref id=id></canvas>
        </div>
    }
}
