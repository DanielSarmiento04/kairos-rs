//! Error boundary component for displaying errors.

use leptos::prelude::*;

/// Displays an error message in a styled container.
/// 
/// # Properties
/// 
/// * `error` - The error message to display
/// * `title` - Optional error title
#[component]
pub fn ErrorBoundaryView(
    /// The error message
    error: String,
    /// Optional error title
    #[prop(optional)]
    title: String,
) -> impl IntoView {
    view! {
        <div class="error-container">
            <div class="error-icon">"⚠️"</div>
            
            <h3 class="error-title">{title}</h3>
            
            <p class="error-message">{error}</p>
            
            <div class="error-actions">
                <button 
                    class="btn btn-primary"
                    on:click=move |_| {
                        // Reload the page
                        #[cfg(feature = "hydrate")]
                        {
                            let _ = web_sys::window().map(|w| w.location().reload());
                        }
                    }
                >
                    "Retry"
                </button>
            </div>
        </div>
    }
}
