mod dashboard;
mod dashboard_simple;
mod routes;
mod metrics;
mod config;
mod health;

// Use the simple dashboard for now
pub use dashboard_simple::Dashboard;
pub use routes::{RoutesPage, MetricsPage, ConfigPage, HealthPage};