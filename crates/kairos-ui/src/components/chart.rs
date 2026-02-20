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

    #[wasm_bindgen(method, getter)]
    fn data(this: &Chart) -> JsValue;

    #[wasm_bindgen(method, setter)]
    fn set_data(this: &Chart, data: JsValue);

    #[wasm_bindgen(method, getter)]
    fn options(this: &Chart) -> JsValue;

    #[wasm_bindgen(method, setter)]
    fn set_options(this: &Chart, options: JsValue);
}

#[component]
pub fn Chart(
    #[prop(into)] id: String,
    #[prop(into)] kind: Signal<String>,
    #[prop(into)] data: Signal<Value>,
    #[prop(into, optional)] options: Option<Signal<Value>>,
) -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    // Use SendWrapper to satisfy StoredValue's Send requirement
    let chart_instance = StoredValue::new(None::<SendWrapper<Chart>>);

    // Effect to initialize/update chart when data changes
    Effect::new(move |_| {
        let data_val = data.get();
        let options_val = options.map(|o| o.get());
        let kind_val = kind.get();

        #[cfg(feature = "hydrate")]
        request_animation_frame(move || {
            if let Some(canvas) = canvas_ref.get() {
                let mut needs_create = true;

                chart_instance.update_value(|chart_opt| {
                    if let Some(wrapper) = chart_opt {
                        if let Ok(data_js) = serde_wasm_bindgen::to_value(&data_val) {
                            wrapper.set_data(data_js);
                        }

                        if let Some(opts) = options_val.clone() {
                            if let Ok(opts_js) = serde_wasm_bindgen::to_value(&opts) {
                                wrapper.set_options(opts_js);
                            }
                        }

                        wrapper.update();
                        needs_create = false;
                    }
                });

                if needs_create {
                    // Prepare config
                    let config = serde_json::json!({
                        "type": kind_val,
                        "data": data_val,
                        "options": options_val.unwrap_or(serde_json::json!({
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
