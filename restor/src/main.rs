#[instrument]
async fn index() -> Html<String> {
    let html = load_template("templates/index.html").await;
    Html(html)
}async fn load_template(file_path: &str) -> String {
    match fs::read_to_string(file_path).await {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to load template {}: {}", file_path, e);
            // Fallback HTML
            format!(
                r#"<!DOCTYPE html>
<html><head><title>Error</title></head>
<body><h1>Template Error</h1><p>Could not load template: {}</p></body></html>"#,
                file_path
            )
        }
    }
}

async fn handle_404() -> impl IntoResponse {
    warn!("404 - Page not found");
    let html = load_template("templates/errors/404.html").await;
    (StatusCode::NOT_FOUND, Html(html))
}

async fn handle_500() -> impl IntoResponse {
    warn!("500 - Internal server error");
    let html = load_template("templates/errors/500.html").await;
    (StatusCode::INTERNAL_SERVER_ERROR, Html(html))
}

async fn handle_405() -> impl IntoResponse {
    warn!("405 - Method not allowed");
    let html = load_template("templates/errors/405.html").await;
    (StatusCode::METHOD_NOT_ALLOWED, Html(html))
}

#[instrument(skip(req, next))]
async fn error_handler(req: Request, next: Next) -> Response {
    let response = next.run(req).await;
    
    match response.status() {
        StatusCode::NOT_FOUND => handle_404().await.into_response(),
        StatusCode::INTERNAL_SERVER_ERROR => handle_500().await.into_response(),
        StatusCode::METHOD_NOT_ALLOWED => handle_405().await.into_response(),
        _ => response,
    }
}use axum::{
    extract::Request,
    http::{StatusCode, HeaderMap},
    middleware::{self, Next},
    response::{Response, Html, IntoResponse},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::time::Instant;
use tokio::{fs, net::TcpListener};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{info, instrument, Level, warn, error};
use tracing_subscriber::{
    fmt::writer::MakeWriterExt,
    prelude::*,
    EnvFilter,
};
use tracing_appender::{non_blocking, rolling};
use std::io;

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
    timestamp: u64,
}

#[derive(Serialize)]
struct ApiError {
    error: String,
    code: u16,
}

#[derive(Serialize)]
struct Message {
    message: String,
    version: String,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

impl ApiResponse<ApiError> {
    fn error(message: String, code: u16) -> Self {
        Self {
            success: false,
            data: ApiError {
                error: message,
                code,
            },
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[instrument]
async fn hello_world() -> Json<ApiResponse<Message>> {
    let message = Message {
        message: "Hello from restor!".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    
    Json(ApiResponse::success(message))
}

#[instrument]
async fn health_check() -> (StatusCode, Json<ApiResponse<&'static str>>) {
    (StatusCode::OK, Json(ApiResponse::success("OK")))
}

#[instrument(skip(req, next))]
async fn log_requests(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = Instant::now();
    
    let response = next.run(req).await;
    let duration = start.elapsed();
    
    info!(
        method = %method,
        path = %path,
        status = %response.status(),
        duration_ms = %duration.as_millis(),
        "Request processed"
    );
    
    response
}

async fn create_app() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/hello", get(hello_world))
        .route("/api/health", get(health_check))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(middleware::from_fn(log_requests))
                .layer(middleware::from_fn(error_handler))
        )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup file logging
    let file_appender = rolling::daily("logs", "restor.log");
    let (non_blocking_file, _guard) = non_blocking(file_appender);
    
    // Setup console logging
    let (non_blocking_stdout, _guard2) = non_blocking(io::stdout());
    
    // Initialize tracing with both file and console output
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking_stdout)
                .with_ansi(true)
                .with_target(false)
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking_file)
                .with_ansi(false)
                .with_target(true)
                .json()
        )
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    let app = create_app().await;
    
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    info!("🚀 restor listening on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_index_page() {
        let app = create_app().await;
        let server = TestServer::new(app).unwrap();
        
        let response = server.get("/").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        assert!(response.text().contains("restor"));
    }

    #[tokio::test]
    async fn test_404_page() {
        let app = create_app().await;
        let server = TestServer::new(app).unwrap();
        
        let response = server.get("/nonexistent").await;
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
        assert!(response.text().contains("404"));
    }

    #[tokio::test]
    async fn test_hello_endpoint() {
        let app = create_app().await;
        let server = TestServer::new(app).unwrap();
        
        let response = server.get("/api/hello").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let body: ApiResponse<Message> = response.json();
        assert!(body.success);
        assert_eq!(body.data.message, "Hello from restor!");
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = create_app().await;
        let server = TestServer::new(app).unwrap();
        
        let response = server.get("/api/health").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let body: ApiResponse<&str> = response.json();
        assert!(body.success);
        assert_eq!(body.data, "OK");
    }
}
