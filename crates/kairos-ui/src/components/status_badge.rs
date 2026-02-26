//! Status badge component for displaying health status indicators.

use leptos::prelude::*;

/// Status badge variant types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatusVariant {
    Success,
    Warning,
    Error,
    Info,
    Neutral,
}

impl StatusVariant {
    pub fn as_class(&self) -> &'static str {
        match self {
            StatusVariant::Success => "status-badge-success",
            StatusVariant::Warning => "status-badge-warning",
            StatusVariant::Error => "status-badge-error",
            StatusVariant::Info => "status-badge-info",
            StatusVariant::Neutral => "status-badge-neutral",
        }
    }
}

/// Displays a colored status badge with text.
///
/// # Properties
///
/// * `text` - The badge text to display
/// * `variant` - The badge color variant
#[component]
pub fn StatusBadge(
    /// The badge text
    text: String,
    /// The badge variant (color)
    variant: StatusVariant,
) -> impl IntoView {
    let badge_class = format!("status-badge {}", variant.as_class());

    view! {
        <span class=badge_class>
            {text}
        </span>
    }
}
