#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use kairos_ui::*;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Build our application with a route  
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, App)
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("🚀 Kairos UI listening on http://{}", &addr);
    axum::serve(listener, app).await.unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    use kairos_ui::*;
    
    console_error_panic_hook::set_once();
    
    leptos::mount_to_body(App);
}