use crate::services::http::RouteHandler;
use actix_web::{web, HttpRequest};

pub fn configure_route(cfg: &mut web::ServiceConfig, handler: RouteHandler) {
    cfg.app_data(web::PayloadConfig::new(1024 * 1024)) // 1MB payload limit (reduced from 10MB)
        .app_data(web::JsonConfig::default().limit(1024 * 1024)) // 1MB JSON limit
        .service(
            web::resource("/{tail:.*}").to(move |req: HttpRequest, body: web::Bytes| {
                let handler: RouteHandler = handler.clone();
                async move { handler.handle_request(req, body).await }
            }),
        );
}
