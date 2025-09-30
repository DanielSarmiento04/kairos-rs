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
    <link rel="stylesheet" href="/pkg/kairos-ui.css">
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
    </style>
</head>
<body>
    <div id="leptos"></div>
    <script type="module">
        import init from '/pkg/kairos-ui.js';
        async function main() {
            await init('/pkg/kairos-ui.wasm');
        }
        main()
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
    
    leptos::mount_to_body(App);
}