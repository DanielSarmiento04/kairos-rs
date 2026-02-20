pub mod dashboard;
pub mod routes_page;
pub mod metrics_page;
pub mod config_page;
pub mod health_page;
pub mod profile_page;

// Re-export pages for easier imports
pub use dashboard::DashboardPage;
pub use routes_page::RoutesPage;
pub use metrics_page::MetricsPage;
pub use config_page::ConfigPage;
pub use health_page::HealthPage;
pub use profile_page::ProfilePage;
