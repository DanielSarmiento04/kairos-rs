#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{Router, response::Html, http::StatusCode};
    use tower_http::services::ServeDir;
    use std::net::SocketAddr;
    
    println!("🚀 Starting Kairos UI server...");
    
    // Simple static file server for the WASM build
    let app = Router::new()
        .nest_service("/", ServeDir::new("target/site"))
        .fallback(|| async { 
            (StatusCode::OK, Html(include_str!("../index.html")))
        });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    println!("🌐 Kairos UI server listening on http://{}", addr);
    println!("📦 Serving WASM application from target/site");
    
    axum::serve(listener, app).await.unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    use kairos_ui::*;
    
    console_error_panic_hook::set_once();
    
    leptos::mount_to_body(App);
}