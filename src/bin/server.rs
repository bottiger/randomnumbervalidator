use axum::{
    extract::Json,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use randomnumbervalidator::{
    validate_random_numbers_with_nist, ValidationRequest, ValidationResponse,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "randomnumbervalidator=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Random Number Validator server");

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/validate", post(validate_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr = "127.0.0.1:3000";
    info!("Server listening on http://{}", addr);
    println!("Server running on http://{}", addr);
    println!("Open http://127.0.0.1:3000 in your browser");
    println!("Set RUST_LOG=debug for detailed logging");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_index() -> impl IntoResponse {
    info!("Serving index page");
    Html(include_str!("../../static/index.html"))
}

async fn validate_handler(Json(payload): Json<ValidationRequest>) -> Json<ValidationResponse> {
    info!(
        "Validation request received: {} numbers, NIST={}",
        payload.numbers.split(',').count(),
        payload.use_nist
    );

    let response = validate_random_numbers_with_nist(&payload.numbers, payload.use_nist);

    if response.valid {
        info!(
            "Validation successful: quality_score={:.2}",
            response.quality_score
        );
    } else {
        warn!(
            "Validation failed: quality_score={:.2}, reason={}",
            response.quality_score, response.message
        );
    }

    Json(response)
}
