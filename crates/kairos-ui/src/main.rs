#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{Router, response::Html, http::StatusCode};
    use tower_http::services::ServeDir;
    use std::net::SocketAddr;
    
    println!("🚀 Starting Kairos UI server...");
    
    // HTML template for the WASM app
    const HTML_CONTENT: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Kairos UI</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: #f5f7fa;
            color: #2d3748;
        }
        
        #leptos {
            min-height: 90vh;
            background: white;
            border-radius: 8px;
            padding: 20px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
        }
        
        .loading {
            text-align: center;
            padding: 40px;
            color: #666;
        }
    </style>
</head>
<body>
    <h1>🔄 Kairos Gateway</h1>
    <div id="leptos">
        <div class="loading">Loading application...</div>
    </div>
    
    <script type="module">
        console.log('Script starting...');
        
        import init from '/pkg/kairos-ui.js';
        
        async function main() {
            try {
                console.log('Initializing WASM...');
                await init('/pkg/kairos-ui.wasm');
                console.log('WASM initialized successfully!');
            } catch (error) {
                console.error('Failed to load WASM:', error);
                document.getElementById('leptos').innerHTML = 
                    '<div class="loading" style="color: red;">❌ Failed to load application: ' + error.message + '</div>';
            }
        }
        
        main();
    </script>
</body>
</html>"#;
    
    // Simple static file server for the WASM build
    let app = Router::new()
        .nest_service("/pkg", ServeDir::new("target/site/pkg"))
        .nest_service("/styles", ServeDir::new("target/site"))
        .fallback(|| async { 
            (StatusCode::OK, Html(HTML_CONTENT))
        });

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    println!("🌐 Kairos UI server listening on http://{}", addr);
    println!("📦 Serving WASM application from target/site");
    println!("🌐 Open http://localhost:3000 in your browser");
    
    axum::serve(listener, app).await.unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    use kairos_ui::*;
    
    console_error_panic_hook::set_once();
    
    // Add some debug logging
    web_sys::console::log_1(&"Starting Leptos app...".into());
    
    // Mount to the specific div instead of body
    leptos::mount_to(
        leptos::document().get_element_by_id("leptos").unwrap(),
        App
    );
    
    web_sys::console::log_1(&"Leptos app mounted to #leptos div!".into());
}