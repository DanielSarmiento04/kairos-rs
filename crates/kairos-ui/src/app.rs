use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod components;
mod pages;
mod services;

use components::*;
use pages::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context for the router
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/kairos-ui.css"/>
        
        // Sets page title
        <Title text="Kairos Gateway Admin"/>
        
        // Sets meta description
        <Meta name="description" content="Admin interface for Kairos API Gateway"/>
        
        <Router>
            <main>
                <Header/>
                <div class="container">
                    <Sidebar/>
                    <div class="main-content">
                        <Routes>
                            <Route path="" view=Dashboard/>
                            <Route path="/routes" view=RoutesPage/>
                            <Route path="/metrics" view=MetricsPage/>
                            <Route path="/config" view=ConfigPage/>
                            <Route path="/health" view=HealthPage/>
                            <Route path="/*any" view=NotFound/>
                        </Routes>
                    </div>
                </div>
            </main>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    // Set an HTTP status code 404
    #[cfg(feature = "ssr")]
    {
        let resp = expect_context::<leptos_axum::ResponseOptions>();
        resp.set_status(axum::http::StatusCode::NOT_FOUND);
    }
    
    view! {
        <div class="error-page">
            <h1>"Page Not Found"</h1>
            <p>"The page you're looking for doesn't exist."</p>
            <a href="/">"Go Home"</a>
        </div>
    }
}