pub mod chart;
pub mod config;
pub mod error_boundary;
pub mod loading;
pub mod metric_card;
pub mod navbar;
pub mod real_time_metrics;
pub mod sidebar;
pub mod status_badge;

// Re-export StatusVariant for use in pages
pub use status_badge::StatusVariant;

// Re-export components for easier imports
pub use chart::Chart;
pub use error_boundary::ErrorBoundaryView;
pub use loading::LoadingSpinner;
pub use metric_card::MetricCard;
pub use navbar::Navbar;
pub use real_time_metrics::RealTimeMetrics;
pub use sidebar::Sidebar;
pub use status_badge::StatusBadge;
