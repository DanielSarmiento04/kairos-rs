//! Enhanced structured logging with JSON output and correlation IDs.
//! 
//! This module extends the existing logging capabilities with structured JSON output,
//! correlation ID tracking, and enhanced metadata for production observability.

use chrono::{DateTime, Utc};
use log::{Level, Record};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Write;
use uuid::Uuid;

/// Structured log entry with correlation information.
/// 
/// This structure represents a single log entry with enhanced metadata for
/// tracing requests through the system and providing rich context for debugging.
#[derive(Debug, Clone)]
pub struct StructuredLogEntry {
    /// Log timestamp in RFC3339 format
    pub timestamp: DateTime<Utc>,
    /// Log level (error, warn, info, debug, trace)
    pub level: String,
    /// Log message
    pub message: String,
    /// Source file where log was generated
    pub file: Option<String>,
    /// Line number in source file
    pub line: Option<u32>,
    /// Module path where log was generated
    pub module: Option<String>,
    /// Correlation ID for request tracing
    pub correlation_id: Option<String>,
    /// Additional structured fields
    pub fields: HashMap<String, Value>,
    /// Service name for multi-service environments
    pub service: String,
    /// Service version
    pub version: String,
}

impl StructuredLogEntry {
    /// Creates a new structured log entry from a log record.
    pub fn from_record(record: &Record, correlation_id: Option<String>) -> Self {
        let mut fields = HashMap::new();
        
        // Add target information
        fields.insert("target".to_string(), json!(record.target()));
        
        Self {
            timestamp: Utc::now(),
            level: record.level().to_string().to_lowercase(),
            message: record.args().to_string(),
            file: record.file().map(|f| f.to_string()),
            line: record.line(),
            module: record.module_path().map(|m| m.to_string()),
            correlation_id,
            fields,
            service: "kairos-rs".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    
    /// Converts the log entry to JSON format.
    pub fn to_json(&self) -> String {
        let mut json_obj = json!({
            "@timestamp": self.timestamp.to_rfc3339(),
            "level": self.level,
            "message": self.message,
            "service": self.service,
            "version": self.version
        });
        
        // Add optional fields
        if let Some(ref file) = self.file {
            json_obj["file"] = json!(file);
        }
        
        if let Some(line) = self.line {
            json_obj["line"] = json!(line);
        }
        
        if let Some(ref module) = self.module {
            json_obj["module"] = json!(module);
        }
        
        if let Some(ref correlation_id) = self.correlation_id {
            json_obj["correlation_id"] = json!(correlation_id);
        }
        
        // Add additional fields
        for (key, value) in &self.fields {
            json_obj[key] = value.clone();
        }
        
        serde_json::to_string(&json_obj).unwrap_or_else(|_| {
            format!(r#"{{"error":"Failed to serialize log entry","message":"{}"}}"#, self.message)
        })
    }
    
    /// Adds a custom field to the log entry.
    pub fn with_field(mut self, key: &str, value: Value) -> Self {
        self.fields.insert(key.to_string(), value);
        self
    }
}

/// Middleware for generating and managing correlation IDs.
/// 
/// This structure provides correlation ID generation and extraction from HTTP requests,
/// enabling request tracing across the entire system.
pub struct CorrelationId;

impl CorrelationId {
    /// Generates a new correlation ID.
    pub fn generate() -> String {
        Uuid::new_v4().to_string()
    }
    
    /// Extracts correlation ID from HTTP request headers.
    /// 
    /// Looks for correlation ID in the following headers (in order):
    /// 1. `X-Correlation-ID`
    /// 2. `X-Request-ID`
    /// 3. `X-Trace-ID`
    /// 
    /// If none found, generates a new correlation ID.
    pub fn from_request(req: &actix_web::HttpRequest) -> String {
        // Try to extract from headers
        let headers = req.headers();
        
        // Check standard correlation ID headers
        for header_name in &["x-correlation-id", "x-request-id", "x-trace-id"] {
            if let Some(header_value) = headers.get(*header_name) {
                if let Ok(id) = header_value.to_str() {
                    if !id.is_empty() {
                        return id.to_string();
                    }
                }
            }
        }
        
        // Generate new correlation ID
        Self::generate()
    }
}

/// Enhanced JSON formatter for structured logging.
/// 
/// This formatter outputs logs in JSON format suitable for log aggregation
/// systems like ELK stack, Splunk, or cloud logging services.
pub struct JsonFormatter;

impl JsonFormatter {
    /// Formats a log record as JSON with correlation context.
    pub fn format(record: &Record, correlation_id: Option<String>) -> String {
        let entry = StructuredLogEntry::from_record(record, correlation_id);
        entry.to_json()
    }
}

/// Enhanced human-readable formatter for development.
/// 
/// This formatter provides rich, colored output for local development with
/// correlation ID information included.
pub struct HumanFormatter;

impl HumanFormatter {
    /// Formats a log record for human reading with optional correlation ID.
    pub fn format(record: &Record, correlation_id: Option<String>) -> String {
        let timestamp = chrono::Local::now().format("%h %d %y %I:%M:%S %p");
        let level = match record.level() {
            Level::Error => "\x1b[31m[ERROR]\x1b[0m", // Red
            Level::Warn => "\x1b[33m[WARN] \x1b[0m",  // Yellow  
            Level::Info => "\x1b[32m[INFO] \x1b[0m",   // Green
            Level::Debug => "\x1b[34m[DEBUG]\x1b[0m", // Blue
            Level::Trace => "\x1b[35m[TRACE]\x1b[0m", // Magenta
        };
        
        let file_line = if let (Some(file), Some(line)) = (record.file(), record.line()) {
            format!("{}:{}", file, line)
        } else {
            "unknown".to_string()
        };
        
        let correlation_part = if let Some(ref corr_id) = correlation_id {
            format!(" [{}]", &corr_id[..8]) // Show first 8 chars of correlation ID
        } else {
            String::new()
        };
        
        format!(
            "{} | {} | {:22} | {}{}",
            timestamp,
            level,
            file_line,
            record.args(),
            correlation_part
        )
    }
}

/// Configures structured logging based on environment variables.
/// 
/// This function sets up either JSON or human-readable logging based on
/// configuration, with support for correlation ID tracking.
/// 
/// # Environment Variables
/// 
/// - `KAIROS_LOG_FORMAT`: "json" for JSON output, "human" for readable output (default: "human")
/// - `KAIROS_LOG_LEVEL`: Log level filter (default: "info")
/// - `RUST_LOG`: Standard Rust log level override
/// 
/// # JSON Format
/// 
/// When JSON format is enabled, logs are output as structured JSON suitable
/// for log aggregation systems:
/// 
/// ```json
/// {
///   "@timestamp": "2024-03-15T10:30:00Z",
///   "level": "info",
///   "message": "Server started successfully",
///   "service": "kairos-rs",
///   "version": "0.2.4",
///   "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
///   "file": "src/main.rs",
///   "line": 100
/// }
/// ```
pub fn configure_enhanced_logger() {
    let log_format = std::env::var("KAIROS_LOG_FORMAT")
        .unwrap_or_else(|_| "human".to_string())
        .to_lowercase();
    
    let log_level = std::env::var("KAIROS_LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase();
    
    let level_filter = match log_level.as_str() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    };
    
    let mut builder = env_logger::Builder::new();
    builder.filter_level(level_filter);
    
    // Override with RUST_LOG if present
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        builder.parse_filters(&rust_log);
    }
    
    match log_format.as_str() {
        "json" => {
            log::info!("Configuring JSON structured logging");
            builder.format(|buf, record| {
                // For now, JSON format without correlation ID
                // In a full implementation, we'd need request context
                let json_output = JsonFormatter::format(record, None);
                writeln!(buf, "{}", json_output)
            });
        },
        _ => {
            log::info!("Configuring human-readable logging");
            builder.format(|buf, record| {
                // Human format without correlation ID for now
                let human_output = HumanFormatter::format(record, None);
                writeln!(buf, "{}", human_output)
            });
        }
    }
    
    builder.init();
    
    log::info!("Enhanced structured logging configured (format: {}, level: {})", log_format, log_level);
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::Record;
    use std::time::SystemTime;

    #[test]
    fn test_correlation_id_generation() {
        let id1 = CorrelationId::generate();
        let id2 = CorrelationId::generate();
        
        assert_ne!(id1, id2);
        assert!(id1.len() > 0);
        assert!(id2.len() > 0);
    }

    #[test]
    fn test_structured_log_entry_creation() {
        let record = Record::builder()
            .args(format_args!("Test message"))
            .level(Level::Info)
            .target("test")
            .file(Some("test.rs"))
            .line(Some(42))
            .module_path(Some("test::module"))
            .build();
        
        let correlation_id = Some("test-correlation-id".to_string());
        let entry = StructuredLogEntry::from_record(&record, correlation_id);
        
        assert_eq!(entry.level, "info");
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.correlation_id, Some("test-correlation-id".to_string()));
        assert_eq!(entry.file, Some("test.rs".to_string()));
        assert_eq!(entry.line, Some(42));
    }

    #[test]
    fn test_json_serialization() {
        let mut entry = StructuredLogEntry {
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
            level: "info".to_string(),
            message: "Test message".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(42),
            module: Some("test".to_string()),
            correlation_id: Some("test-id".to_string()),
            fields: HashMap::new(),
            service: "kairos-rs".to_string(),
            version: "0.1.0".to_string(),
        };
        
        entry.fields.insert("custom_field".to_string(), json!("custom_value"));
        
        let json_str = entry.to_json();
        let parsed: Value = serde_json::from_str(&json_str).unwrap();
        
        assert_eq!(parsed["level"], "info");
        assert_eq!(parsed["message"], "Test message");
        assert_eq!(parsed["correlation_id"], "test-id");
        assert_eq!(parsed["custom_field"], "custom_value");
    }
}