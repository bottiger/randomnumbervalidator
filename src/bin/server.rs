use axum::{
    extract::{ConnectInfo, Json, State},
    http::HeaderMap,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use randomnumbervalidator::{
    validate_random_numbers_full, ValidationRequest, ValidationResponse,
};
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
        "Validation request received: query_id={}, ip={}, {} numbers, NIST={}",
        query_id,
        client_ip,
        payload.numbers.split(',').count(),
        payload.use_nist
    );

    // Perform validation
    let response = validate_random_numbers_full(
        &payload.numbers,
        payload.use_nist,
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

/// Log query information to the database
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
    let total_bits_count = total_numbers_count * 32; // Each u32 is 32 bits

    // Parse NIST results for summary metrics
    let (nist_tests_passed, nist_tests_total, nist_avg_p_value) =
        parse_nist_summary(response.nist_results.as_deref());

    sqlx::query(
        r#"
        INSERT INTO query_logs (
            query_id, created_at, client_ip, user_agent, country,
            numbers_sample, numbers_truncated, total_numbers_count, total_bits_count,
            valid, quality_score, nist_used,
            nist_tests_passed, nist_tests_total, nist_avg_p_value,
            processing_time_ms, error_message
        ) VALUES (
            $1, NOW(), $2, $3, NULL,
            $4, $5, $6, $7,
            $8, $9, $10,
            $11, $12, $13,
            $14, NULL
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
    .bind(request.use_nist)
    .bind(nist_tests_passed)
    .bind(nist_tests_total)
    .bind(nist_avg_p_value)
    .bind(processing_time_ms)
    .execute(pool)
    .await?;

    info!("Query logged to database: query_id={}", query_id);
    Ok(())
}

/// Parse NIST results to extract summary metrics
fn parse_nist_summary(nist_results: Option<&str>) -> (Option<i32>, Option<i32>, Option<f64>) {
    if let Some(results) = nist_results {
        // Try to parse the summary line that looks like: "Overall: 142/188 tests passed (75.5%)"
        if let Some(line) = results.lines().find(|l| l.contains("tests passed")) {
            let parts: Vec<&str> = line.split_whitespace().collect();

            // Look for pattern "X/Y tests passed"
            for (i, part) in parts.iter().enumerate() {
                if part.contains('/') && i + 2 < parts.len() && parts[i + 1] == "tests" {
                    if let Some((passed, total)) = part.split_once('/') {
                        let passed_count = passed.parse::<i32>().ok();
                        let total_count = total.parse::<i32>().ok();

                        // Try to extract percentage if available
                        let avg_p = if i + 3 < parts.len() {
                            let pct_str =
                                parts[i + 3].trim_matches(|c| c == '(' || c == ')' || c == '%');
                            pct_str.parse::<f64>().ok().map(|v| v / 100.0)
                        } else {
                            None
                        };

                        return (passed_count, total_count, avg_p);
                    }
                }
            }
        }
    }
    (None, None, None)
}
