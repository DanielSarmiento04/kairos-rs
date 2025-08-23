use actix_web::{
    http::{Method as ActixMethod, StatusCode},
    web, App, Error as ActixError, HttpRequest, HttpResponse, HttpServer,
};
use crate::models::http::{RouteHandler, format_route};


pub fn configure_route(cfg: &mut web::ServiceConfig, handler: RouteHandler) {
    cfg.app_data(web::PayloadConfig::new(1024 * 1024 * 10)) // 10MB payload limit
        .service(
            web::resource("/{tail:.*}").to(move |req: HttpRequest, body: web::Bytes| {
                let handler: RouteHandler = handler.clone();
                async move { handler.handle_request(req, body).await }
            }),
        );
}