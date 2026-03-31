pub mod config_page;
pub mod dashboard;
pub mod health_page;
pub mod metrics_page;
pub mod routes_page;

// Re-export pages for easier imports
pub use config_page::ConfigPage;
pub use dashboard::DashboardPage;
pub use health_page::HealthPage;
pub use metrics_page::MetricsPage;
pub use routes_page::RoutesPage;
