use axum::{
    extract::Json,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use randomnumbervalidator::{validate_random_numbers, ValidationRequest, ValidationResponse};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/validate", post(validate_handler))
        .layer(CorsLayer::permissive());

    let addr = "127.0.0.1:3000";
    println!("Server running on http://{}", addr);
    println!("Open http://127.0.0.1:3000 in your browser");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_index() -> impl IntoResponse {
    Html(include_str!("../../static/index.html"))
}

async fn validate_handler(
    Json(payload): Json<ValidationRequest>,
) -> Json<ValidationResponse> {
    let response = validate_random_numbers(&payload.numbers);
    Json(response)
}
