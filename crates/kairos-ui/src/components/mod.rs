pub mod navbar;
pub mod sidebar;
pub mod metric_card;
pub mod status_badge;
pub mod loading;
pub mod error_boundary;

// Re-export StatusVariant for use in pages
pub use status_badge::StatusVariant;

// Re-export components for easier imports
pub use navbar::Navbar;
pub use sidebar::Sidebar;
pub use metric_card::MetricCard;
pub use status_badge::StatusBadge;
pub use loading::LoadingSpinner;
pub use error_boundary::ErrorBoundaryView;
