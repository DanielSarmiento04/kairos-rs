//! Advanced configuration validation with detailed error reporting.
//! 
//! This module provides comprehensive validation for gateway configuration,
//! including security checks, performance recommendations, and detailed
//! error reporting for troubleshooting.

use crate::models::settings::Settings;
use log::{warn, info};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    pub fn add_recommendation(&mut self, recommendation: String) {
        self.recommendations.push(recommendation);
    }
}

/// Enhanced configuration validator with security and performance checks
pub struct ConfigValidator;

impl ConfigValidator {
    /// Performs comprehensive validation of gateway settings
    pub fn validate_comprehensive(settings: &Settings) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Basic validation
        Self::validate_basic_structure(settings, &mut result);
        
        // Security validation
        Self::validate_security(settings, &mut result);
        
        // Performance validation
        Self::validate_performance(settings, &mut result);
        
        // Route conflicts
        Self::validate_route_conflicts(settings, &mut result);
        
        // Log results
        Self::log_validation_results(&result);
        
        result
    }
    
    fn validate_basic_structure(settings: &Settings, result: &mut ValidationResult) {
        if settings.routers.is_empty() {
            result.add_error("No routers configured - gateway will not handle any requests".to_string());
        }
        
        for (index, router) in settings.routers.iter().enumerate() {
            if let Err(error) = router.validate() {
                result.add_error(format!("Router {} validation failed: {}", index, error));
            }
        }
    }
    
    fn validate_security(settings: &Settings, result: &mut ValidationResult) {
        let mut http_routes = 0;
        let mut https_routes = 0;
        
        for router in &settings.routers {
            if router.host.starts_with("http://") {
                http_routes += 1;
                if router.host.contains("localhost") || router.host.contains("127.0.0.1") {
                    result.add_warning(format!(
                        "HTTP route to localhost detected: {} - consider HTTPS for production", 
                        router.host
                    ));
                } else {
                    result.add_warning(format!(
                        "Insecure HTTP route detected: {} - consider HTTPS", 
                        router.host
                    ));
                }
            } else if router.host.starts_with("https://") {
                https_routes += 1;
            }
            
            // Check for overly permissive methods
            if router.methods.len() > 4 {
                result.add_warning(format!(
                    "Route {} allows many HTTP methods ({}) - consider restricting for security",
                    router.external_path, router.methods.len()
                ));
            }
            
            // Check for dangerous paths
            if router.external_path.contains("..") || router.internal_path.contains("..") {
                result.add_error(format!(
                    "Path traversal detected in route: {} -> {}",
                    router.external_path, router.internal_path
                ));
            }
        }
        
        if http_routes > 0 && https_routes == 0 {
            result.add_warning("All routes use HTTP - consider HTTPS for production security".to_string());
        }
    }
    
    fn validate_performance(settings: &Settings, result: &mut ValidationResult) {
        let dynamic_routes = settings.routers.iter()
            .filter(|r| r.external_path.contains('{'))
            .count();
        let static_routes = settings.routers.len() - dynamic_routes;
        
        if dynamic_routes > 50 {
            result.add_warning(format!(
                "High number of dynamic routes ({}) may impact performance - consider route optimization",
                dynamic_routes
            ));
        }
        
        if static_routes == 0 && dynamic_routes > 0 {
            result.add_recommendation(
                "Consider adding static routes for frequently accessed endpoints to improve performance".to_string()
            );
        }
        
        // Check for complex patterns
        for router in &settings.routers {
            let param_count = router.external_path.matches('{').count();
            if param_count > 3 {
                result.add_warning(format!(
                    "Route {} has many parameters ({}) - may impact matching performance",
                    router.external_path, param_count
                ));
            }
        }
    }
    
    fn validate_route_conflicts(settings: &Settings, result: &mut ValidationResult) {
        let mut seen_paths = HashSet::new();
        let mut potential_conflicts = Vec::new();
        
        for router in &settings.routers {
            if seen_paths.contains(&router.external_path) {
                result.add_error(format!(
                    "Duplicate route path detected: {}",
                    router.external_path
                ));
            }
            seen_paths.insert(&router.external_path);
            
            // Check for potential conflicts between static and dynamic routes
            for other_router in &settings.routers {
                if router.external_path != other_router.external_path {
                    if Self::routes_may_conflict(&router.external_path, &other_router.external_path) {
                        potential_conflicts.push((
                            router.external_path.clone(),
                            other_router.external_path.clone()
                        ));
                    }
                }
            }
        }
        
        for (route1, route2) in potential_conflicts {
            result.add_warning(format!(
                "Potential route conflict between '{}' and '{}' - order matters",
                route1, route2
            ));
        }
    }
    
    fn routes_may_conflict(path1: &str, path2: &str) -> bool {
        // Simple heuristic: if one is static and matches the pattern of a dynamic route
        let path1_segments: Vec<&str> = path1.split('/').collect();
        let path2_segments: Vec<&str> = path2.split('/').collect();
        
        if path1_segments.len() != path2_segments.len() {
            return false;
        }
        
        for (seg1, seg2) in path1_segments.iter().zip(path2_segments.iter()) {
            if seg1.starts_with('{') || seg2.starts_with('{') {
                continue; // Parameter segment, could match
            }
            if seg1 != seg2 {
                return false; // Different static segments
            }
        }
        
        // If we get here, routes could potentially conflict
        true
    }
    
    fn log_validation_results(result: &ValidationResult) {
        if result.is_valid {
            info!("Configuration validation passed");
        } else {
            for error in &result.errors {
                log::error!("Validation error: {}", error);
            }
        }
        
        for warning in &result.warnings {
            warn!("Validation warning: {}", warning);
        }
        
        for recommendation in &result.recommendations {
            info!("Recommendation: {}", recommendation);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::router::Router;

    fn create_test_router(host: &str, external_path: &str, methods: Vec<&str>) -> Router {
        Router {
            host: host.to_string(),
            port: 80,
            external_path: external_path.to_string(),
            internal_path: "/test".to_string(),
            methods: methods.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_empty_configuration() {
        let settings = Settings {
            version: 1,
            routers: vec![],
        };
        
        let result = ConfigValidator::validate_comprehensive(&settings);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("No routers configured")));
    }

    #[test]
    fn test_security_warnings() {
        let settings = Settings {
            version: 1,
            routers: vec![
                create_test_router("http://example.com", "/api/test", vec!["GET"]),
            ],
        };
        
        let result = ConfigValidator::validate_comprehensive(&settings);
        assert!(result.warnings.iter().any(|w| w.contains("HTTP route")));
    }

    #[test]
    fn test_performance_warnings() {
        let settings = Settings {
            version: 1,
            routers: vec![
                create_test_router("https://example.com", "/api/{a}/{b}/{c}/{d}", vec!["GET"]),
            ],
        };
        
        let result = ConfigValidator::validate_comprehensive(&settings);
        assert!(result.warnings.iter().any(|w| w.contains("many parameters")));
    }
}