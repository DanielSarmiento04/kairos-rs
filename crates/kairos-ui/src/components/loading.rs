//! Loading spinner component.

use leptos::prelude::*;

/// Displays a loading spinner with optional message.
///
/// # Properties
///
/// * `message` - Optional loading message to display
#[component]
pub fn LoadingSpinner(
    /// Optional loading message
    #[prop(optional)]
    message: String,
) -> impl IntoView {
    view! {
        <div class="loading-container">
            <div class="loading-spinner"></div>
            <p class="loading-message">{message}</p>
        </div>
    }
}
