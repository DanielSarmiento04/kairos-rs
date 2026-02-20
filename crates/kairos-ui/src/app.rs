//! Main application component with routing and layout.

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Stylesheet, Title, Meta};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use crate::components::{Navbar, Sidebar};
use crate::pages::*;

/// Main application component with navigation and routing.
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // Inject stylesheet
        <Stylesheet id="leptos" href="/pkg/kairos-ui.css"/>
        
        // Set document title
        <Title text="Kairos Gateway - Admin UI"/>
        
        // Meta tags for SEO and responsiveness
        <Meta name="description" content="Kairos Gateway Admin Interface - Real-time monitoring and configuration"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        
        // Main router
        <Router>
            // Top navigation bar
            <Navbar />
            
            // Main content area with sidebar
            <main>
                // Left sidebar navigation
                <Sidebar />
                
                // Content area with routes
                <div class="content">
                    <Routes fallback=move || view! { <NotFound /> }>
                        <Route path=StaticSegment("") view=DashboardPage />
                        <Route path=StaticSegment("routes") view=RoutesPage />
                        <Route path=StaticSegment("metrics") view=MetricsPage />
                        <Route path=StaticSegment("config") view=ConfigPage />
                        <Route path=StaticSegment("health") view=HealthPage />
                        <Route path=StaticSegment("profile") view=ProfilePage />
                    </Routes>
                </div>
            </main>
        </Router>
    }
}

/// 404 - Not Found page
#[component]
fn NotFound() -> impl IntoView {
    // Set HTTP status code 404 during SSR
    #[cfg(feature = "ssr")]
    {
        use leptos_actix::ResponseOptions;
        let resp = expect_context::<ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <div class="error-container">
            <div class="error-icon">"🔍"</div>
            <h1 class="error-title">"404 - Page Not Found"</h1>
            <p class="error-message">"The page you're looking for doesn't exist."</p>
            <div class="error-actions">
                <a href="/" class="btn btn-primary">"Go to Dashboard"</a>
            </div>
        </div>
    }
}
