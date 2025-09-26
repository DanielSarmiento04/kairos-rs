use leptos::*;

pub mod dashboard_simple;
pub mod routes;
pub mod metrics;
pub mod config;
pub mod health;

pub use dashboard_simple::Dashboard;
pub use routes::*;
pub use metrics::*;
pub use config::*;
pub use health::*;