//! Logging configuration and structured output for the kairos-rs gateway.
//! 
//! This module provides comprehensive logging capabilities with structured output,
//! configurable log levels, and optional color formatting. The logging system
//! is designed for both development and production environments with performance
//! and observability in mind.
//! 
//! # Module Organization
//! 
//! - [`logger`] - Logger configuration, formatting, and initialization
//! 
//! # Logging Architecture
//! 
//! The logging system provides structured output with these components:
//! 
//! ```text
//! Application Events → Logger → Formatter → Output Stream
//!        ↓               ↓         ↓           ↓
//!   Function Calls   Log Levels  Structured  Console/File
//!   Error Events     Filtering   Format      Destinations
//!   Performance      Categories  Timestamps  Storage
//! ```
//! 
//! # Log Format
//! 
//! The structured log format includes:
//! - **Timestamp**: High-precision timestamps with timezone
//! - **Log Level**: Colored level indicators (ERROR, WARN, INFO, DEBUG, TRACE)
//! - **Source Location**: File and line number for debugging
//! - **Message**: Structured log message with context
//! - **Request Context**: Request IDs and tracing information
//! 
//! # Log Levels
//! 
//! - **ERROR**: Critical errors that require immediate attention
//! - **WARN**: Warning conditions that should be investigated
//! - **INFO**: General information about application operation
//! - **DEBUG**: Detailed debugging information for development
//! - **TRACE**: Very detailed tracing information for deep debugging
//! 
//! # Configuration
//! 
//! ## Environment Variables
//! - `RUST_LOG`: Controls log level filtering (e.g., `debug`, `info`, `warn`)
//! - `NO_COLOR`: Disables colored output for structured logging systems
//! 
//! ## Color Output
//! - **Automatic Detection**: Detects terminal capabilities for color support
//! - **Color Coding**: Different colors for each log level for easy identification
//! - **Production Safe**: Honors `NO_COLOR` environment variable
//! 
//! # Performance Features
//! 
//! - **Efficient Formatting**: Optimized string operations with minimal allocations
//! - **Level Filtering**: Early filtering prevents unnecessary processing
//! - **Async Safe**: Compatible with async runtime and multi-threaded environments
//! - **Memory Efficient**: Bounded memory usage with efficient buffer management
//! 
//! # Examples
//! 
//! ```rust
//! use kairos_rs::logs::logger::configure_logger;
//! use log::{info, warn, error, debug, trace};
//! 
//! // Initialize logging system
//! configure_logger();
//! 
//! // Use structured logging throughout application
//! info!("Gateway starting on port {}", 5900);
//! warn!("Configuration validation warning: {}", warning_msg);
//! error!("Failed to connect to upstream: {}", error);
//! debug!("Route matched: {} -> {}", external_path, internal_path);
//! trace!("Request headers: {:?}", headers);
//! ```
//! 
//! # Production Considerations
//! 
//! - **Log Rotation**: Integrate with external log rotation tools
//! - **Centralized Logging**: Compatible with structured log aggregation systems
//! - **Performance Impact**: Minimal overhead in production with appropriate levels
//! - **Security**: Sensitive information filtering and sanitization
//! 
//! # Integration Points
//! 
//! The logging system integrates with:
//! - **Error Handling**: Automatic error logging with context
//! - **Request Tracing**: Request lifecycle logging with IDs
//! - **Performance Monitoring**: Request timing and throughput metrics
//! - **Security Auditing**: Security event logging and alerting

pub mod logger;
