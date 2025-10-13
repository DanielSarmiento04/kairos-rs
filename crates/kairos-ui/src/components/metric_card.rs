//! Metric card component for displaying dashboard statistics.

use leptos::prelude::*;

/// Displays a single metric in a card format.
/// 
/// # Properties
/// 
/// * `title` - The metric title/label
/// * `value` - The main metric value to display
/// * `subtitle` - Optional subtitle or description
/// * `trend` - Optional trend indicator ("up", "down", or "neutral")
/// * `icon` - Optional emoji icon
#[component]
pub fn MetricCard(
    /// The metric title
    title: String,
    /// The main metric value
    value: String,
    /// Optional subtitle
    #[prop(optional)]
    subtitle: Option<String>,
    /// Optional trend indicator
    #[prop(optional)]
    trend: Option<String>,
    /// Optional icon
    #[prop(optional)]
    icon: Option<String>,
) -> impl IntoView {
    let trend_for_class = trend.clone();
    let trend_for_arrow = trend.clone();
    let trend_clone = trend.clone();

    let trend_class = move || {
        match trend_for_class.clone().as_deref() {
            Some("up") => "metric-trend metric-trend-up",
            Some("down") => "metric-trend metric-trend-down",
            _ => "metric-trend metric-trend-neutral",
        }
    };
    
    
    view! {
        <div class="metric-card">
            {icon.map(|i| view! {
                <div class="metric-icon">{i}</div>
            })}
            
            <div class="metric-content">
                <h3 class="metric-title">{title}</h3>
                <p class="metric-value">{value}</p>
                
                {subtitle.map(|sub| view! {
                    <p class="metric-subtitle">{sub}</p>
                })}
                
                {trend_clone.as_ref().map(|_| view! {
                    <div class=trend_class>
                        {move || match trend_for_arrow.clone().as_deref() {
                            Some("up") => "↑",
                            Some("down") => "↓",
                            _ => "→",
                        }}
                    </div>
                })}
            </div>
        </div>
    }
}
