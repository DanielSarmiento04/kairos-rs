//! FTP proxy route handling for the kairos-rs gateway.
//!
//! This module provides HTTP endpoints that proxy FTP operations to
//! backend FTP servers, enabling FTP functionality through HTTP APIs.

use crate::models::router::Backend;
use crate::services::ftp::FtpHandler;
use actix_web::{web, Error, HttpResponse};
use base64::{engine::general_purpose, Engine};
use log::info;
use serde::{Deserialize, Serialize};

/// Request body for FTP file upload operations.
#[derive(Deserialize)]
pub struct FtpUploadRequest {
    /// FTP username for authentication
    pub username: String,
    /// FTP password for authentication
    pub password: String,
    /// Target file path on FTP server
    pub file_path: String,
    /// File content as base64-encoded string
    pub content: String,
}

/// Request parameters for FTP operations.
#[derive(Deserialize)]
pub struct FtpRequest {
    /// FTP username for authentication
    pub username: String,
    /// FTP password for authentication
    pub password: String,
    /// Optional path for operations (defaults to /)
    #[serde(default = "default_path")]
    pub path: String,
}

fn default_path() -> String {
    "/".to_string()
}

/// Response for FTP directory listing.
#[derive(Serialize)]
pub struct FtpListResponse {
    /// List of files and directories
    pub files: Vec<String>,
    /// Number of items
    pub count: usize,
}

/// Response for FTP file download.
#[derive(Serialize)]
pub struct FtpDownloadResponse {
    /// File path
    pub file_path: String,
    /// File content as base64-encoded string
    pub content: String,
    /// File size in bytes
    pub size: usize,
}

/// Handles FTP directory listing requests.
///
/// # Request
///
/// ```json
/// GET /ftp/list?username=user&password=pass&path=/pub
/// ```
///
/// # Response
///
/// ```json
/// {
///   "files": ["file1.txt", "file2.pdf", "dir/"],
///   "count": 3
/// }
/// ```
pub async fn handle_ftp_list(
    query: web::Query<FtpRequest>,
    handler: web::Data<FtpHandler>,
    backend: web::Data<Backend>,
) -> Result<HttpResponse, Error> {
    info!("FTP LIST request for path: {}", query.path);

    let files = handler
        .list_directory(&backend, &query.username, &query.password, &query.path)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let count = files.len();

    Ok(HttpResponse::Ok().json(FtpListResponse { files, count }))
}

/// Handles FTP file download requests.
///
/// # Request
///
/// ```json
/// GET /ftp/download?username=user&password=pass&path=/file.txt
/// ```
///
/// # Response
///
/// ```json
/// {
///   "file_path": "/file.txt",
///   "content": "SGVsbG8gV29ybGQh",
///   "size": 12
/// }
/// ```
pub async fn handle_ftp_download(
    query: web::Query<FtpRequest>,
    handler: web::Data<FtpHandler>,
    backend: web::Data<Backend>,
) -> Result<HttpResponse, Error> {
    info!("FTP RETR request for file: {}", query.path);

    let content = handler
        .retrieve_file(&backend, &query.username, &query.password, &query.path)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let size = content.len();
    // Base64 encode for JSON transport
    let encoded_content = general_purpose::STANDARD.encode(&content);

    Ok(HttpResponse::Ok().json(FtpDownloadResponse {
        file_path: query.path.clone(),
        content: encoded_content,
        size,
    }))
}

/// Handles FTP file upload requests.
///
/// # Request
///
/// ```json
/// POST /ftp/upload
/// {
///   "username": "user",
///   "password": "pass",
///   "file_path": "/upload/file.txt",
///   "content": "SGVsbG8gV29ybGQh"
/// }
/// ```
///
/// # Response
///
/// ```json
/// {
///   "success": true,
///   "message": "File uploaded successfully"
/// }
/// ```
pub async fn handle_ftp_upload(
    body: web::Json<FtpUploadRequest>,
    handler: web::Data<FtpHandler>,
    backend: web::Data<Backend>,
) -> Result<HttpResponse, Error> {
    info!("FTP STOR request for file: {}", body.file_path);

    // Decode base64 content
    let content = general_purpose::STANDARD
        .decode(&body.content)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid base64 content: {}", e)))?;

    handler
        .store_file(
            &backend,
            &body.username,
            &body.password,
            &body.file_path,
            &content,
        )
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "File uploaded successfully"
    })))
}

/// Configures FTP proxy routes for the application.
///
/// Sets up HTTP endpoints that proxy FTP operations:
/// - GET /ftp/list - Directory listing
/// - GET /ftp/download - File download
/// - POST /ftp/upload - File upload
///
/// # Parameters
///
/// * `cfg` - Actix Web service configuration
/// * `handler` - FTP handler instance for processing operations
/// * `backend` - Target FTP server configuration
///
/// # Examples
///
/// ```rust
/// use actix_web::{App, web};
/// use kairos_rs::routes::ftp::configure_ftp;
/// use kairos_rs::services::ftp::FtpHandler;
/// use kairos_rs::models::router::Backend;
///
/// let handler = FtpHandler::new(30);
/// let backend = Backend {
///     host: "ftp://ftp.example.com".to_string(),
///     port: 21,
///     weight: 1,
///     health_check_path: None,
/// };
///
/// let app = App::new()
///     .app_data(web::Data::new(handler))
///     .app_data(web::Data::new(backend))
///     .configure(configure_ftp);
/// ```
pub fn configure_ftp(cfg: &mut web::ServiceConfig) {
    info!("Configuring FTP proxy routes");

    cfg.service(
        web::scope("/ftp")
            .route("/list", web::get().to(handle_ftp_list))
            .route("/download", web::get().to(handle_ftp_download))
            .route("/upload", web::post().to(handle_ftp_upload)),
    );
}
