use axum::{
    extract::{ConnectInfo, Json, State},
    http::HeaderMap,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use randomnumbervalidator::{validate_random_numbers_full, ValidationRequest, ValidationResponse};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::SocketAddr;
use std::time::Instant;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing::{error, info, warn};
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

    // Set up database connection pool (optional - will work without database)
    let database_url = std::env::var("DATABASE_URL").ok();

    let pool = if let Some(url) = database_url {
        info!("Connecting to database...");
        match PgPoolOptions::new().max_connections(5).connect(&url).await {
            Ok(pool) => {
                info!("Database connection established");
                // Run migrations
                match sqlx::migrate!("./migrations").run(&pool).await {
                    Ok(_) => info!("Database migrations completed"),
                    Err(e) => warn!("Failed to run migrations: {}", e),
                }
                Some(pool)
            }
            Err(e) => {
                warn!("Failed to connect to database: {}", e);
                warn!("Continuing without database logging");
                None
            }
        }
    } else {
        info!("DATABASE_URL not set, database logging disabled");
        None
    };

    // Allow configuring host via environment variable for Docker compatibility
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);

    info!("Server listening on http://{}", addr);
    println!("Server running on http://{}", addr);
    println!("Set RUST_LOG=debug for detailed logging");
    if pool.is_some() {
        println!("Database logging enabled");
    } else {
        println!("Database logging disabled (set DATABASE_URL to enable)");
    }

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/game", get(serve_game))
        .route("/api/validate", post(validate_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn serve_index() -> impl IntoResponse {
    info!("Serving index page");

    // Inject git revision info into HTML
    let html = include_str!("../../static/index.html");
    let git_hash = env!("GIT_HASH");
    let git_date = env!("GIT_DATE");

    let html_with_version = html
        .replace("{{GIT_HASH}}", git_hash)
        .replace("{{GIT_DATE}}", git_date);

    Html(html_with_version)
}

async fn serve_game() -> impl IntoResponse {
    info!("Serving game page");

    // Inject git revision info into HTML
    let html = include_str!("../../static/game.html");
    let git_hash = env!("GIT_HASH");
    let git_date = env!("GIT_DATE");

    let html_with_version = html
        .replace("{{GIT_HASH}}", git_hash)
        .replace("{{GIT_DATE}}", git_date);

    Html(html_with_version)
}

async fn validate_handler(
    State(pool): State<Option<PgPool>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<ValidationRequest>,
) -> Json<ValidationResponse> {
    let start_time = Instant::now();
    let query_id = uuid::Uuid::new_v4();

    // Extract client information
    let client_ip = extract_client_ip(&headers, addr);
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown");

    info!(
        "Validation request received: query_id={}, ip={}, {} numbers",
        query_id,
        client_ip,
        payload.numbers.split(',').count()
    );

    // Perform validation (always uses NIST)
    let response = validate_random_numbers_full(
        &payload.numbers,
        &payload.input_format,
        payload.range_min,
        payload.range_max,
        payload.bit_width,
        payload.debug_log,
    );
    let processing_time_ms = start_time.elapsed().as_millis() as i32;

    // Log results
    if response.valid {
        info!(
            "Validation successful: query_id={}, quality_score={:.2}, time={}ms",
            query_id, response.quality_score, processing_time_ms
        );
    } else {
        warn!(
            "Validation failed: query_id={}, quality_score={:.2}, reason={}, time={}ms",
            query_id, response.quality_score, response.message, processing_time_ms
        );
    }

    // Log to database if available
    if let Some(pool) = pool {
        if let Err(e) = log_query_to_database(
            &pool,
            query_id,
            &client_ip,
            user_agent,
            &payload,
            &response,
            processing_time_ms,
        )
        .await
        {
            error!("Failed to log query to database: {}", e);
        }
    }

    Json(response)
}

/// Extract real client IP from headers (considering proxies) or fallback to socket address
fn extract_client_ip(headers: &HeaderMap, addr: SocketAddr) -> String {
    // Check for common proxy headers in order of preference
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(value) = forwarded_for.to_str() {
            // X-Forwarded-For can contain multiple IPs, take the first one
            if let Some(ip) = value.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            return value.to_string();
        }
    }

    // Fallback to socket address
    addr.ip().to_string()
}

/// Log query information to the database using the normalized schema
async fn log_query_to_database(
    pool: &PgPool,
    query_id: uuid::Uuid,
    client_ip: &str,
    user_agent: &str,
    request: &ValidationRequest,
    response: &ValidationResponse,
    processing_time_ms: i32,
) -> Result<(), sqlx::Error> {
    // Prepare sample (first 5KB)
    const MAX_SAMPLE_SIZE: usize = 5 * 1024;
    let numbers_sample = if request.numbers.len() > MAX_SAMPLE_SIZE {
        &request.numbers[..MAX_SAMPLE_SIZE]
    } else {
        &request.numbers
    };
    let numbers_truncated = request.numbers.len() > MAX_SAMPLE_SIZE;

    // Count numbers and calculate total bits
    let total_numbers_count = request
        .numbers
        .split(|c: char| !c.is_numeric())
        .filter(|s| !s.is_empty())
        .count() as i32;

    // Get actual bit count from NIST data if available
    let total_bits_count = if let Some(ref nist_data) = response.nist_data {
        nist_data.bit_count as i32
    } else {
        total_numbers_count * 32 // Fallback estimate
    };

    // Insert into queries table (NIST is always used now)
    sqlx::query(
        r#"
        INSERT INTO queries (
            query_id, created_at, client_ip, user_agent, country,
            numbers_sample, numbers_truncated, total_numbers_count, total_bits_count,
            valid, quality_score, nist_used,
            processing_time_ms, error_message
        ) VALUES (
            $1, NOW(), $2, $3, NULL,
            $4, $5, $6, $7,
            $8, $9, true,
            $10, NULL
        )
        "#,
    )
    .bind(query_id)
    .bind(client_ip)
    .bind(user_agent)
    .bind(numbers_sample)
    .bind(numbers_truncated)
    .bind(total_numbers_count)
    .bind(total_bits_count)
    .bind(response.valid)
    .bind(response.quality_score)
    .bind(processing_time_ms)
    .execute(pool)
    .await?;

    // Insert individual test results if available
    if let Some(ref nist_data) = response.nist_data {
        for test_result in &nist_data.individual_tests {
            if let Err(e) = log_test_result_to_database(pool, query_id, test_result).await {
                warn!(
                    "Failed to log test result '{}' for query {}: {}",
                    test_result.name, query_id, e
                );
            }
        }
    }

    info!("Query logged to database: query_id={}", query_id);
    Ok(())
}

/// Log an individual test result to the database
async fn log_test_result_to_database(
    pool: &PgPool,
    query_id: uuid::Uuid,
    test_result: &randomnumbervalidator::NistTestResult,
) -> Result<(), sqlx::Error> {
    // First, ensure the test definition exists (get or create)
    let test_id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO test_definitions (test_name, description)
        VALUES ($1, $2)
        ON CONFLICT (test_name) DO UPDATE SET test_name = EXCLUDED.test_name
        RETURNING id
        "#,
    )
    .bind(&test_result.name)
    .bind(&test_result.description)
    .fetch_one(pool)
    .await?;

    // Convert p_values Vec to JSON
    let p_values_json = serde_json::to_value(&test_result.p_values)
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    // Convert metrics Option<Vec<(String, String)>> to JSON
    let metrics_json = if let Some(ref metrics) = test_result.metrics {
        serde_json::to_value(metrics).map_err(|e| sqlx::Error::Decode(Box::new(e)))?
    } else {
        serde_json::Value::Null
    };

    // Insert test result
    sqlx::query(
        r#"
        INSERT INTO test_results (query_id, test_id, passed, p_value, p_values, metrics)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (query_id, test_id) DO UPDATE SET
            passed = EXCLUDED.passed,
            p_value = EXCLUDED.p_value,
            p_values = EXCLUDED.p_values,
            metrics = EXCLUDED.metrics
        "#,
    )
    .bind(query_id)
    .bind(test_id)
    .bind(test_result.passed)
    .bind(test_result.p_value)
    .bind(p_values_json)
    .bind(metrics_json)
    .execute(pool)
    .await?;

    Ok(())
}
