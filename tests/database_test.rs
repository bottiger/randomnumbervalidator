/// Integration tests for database operations with the normalized schema
///
/// These tests require a PostgreSQL database to be running.
/// Set DATABASE_URL environment variable to run these tests, or they will be skipped.
///
/// Example: DATABASE_URL=postgres://localhost/randomnumbervalidator_test cargo test
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

/// Helper function to create a test database pool
/// Returns None if DATABASE_URL is not set (tests will be skipped)
async fn create_test_pool() -> Option<PgPool> {
    // Skip tests if DATABASE_URL is not set
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("DATABASE_URL not set, skipping database integration tests");
            return None;
        }
    };

    let pool = match PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to connect to database: {}, skipping tests", e);
            return None;
        }
    };

    // Run migrations
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("Failed to run migrations: {}, skipping tests", e);
        return None;
    }

    Some(pool)
}

/// Helper function to clean up test data
async fn cleanup_test_data(pool: &PgPool, query_id: Uuid) {
    // Delete test results first (due to foreign key)
    sqlx::query("DELETE FROM test_results WHERE query_id = $1")
        .bind(query_id)
        .execute(pool)
        .await
        .ok();

    // Delete query
    sqlx::query("DELETE FROM queries WHERE query_id = $1")
        .bind(query_id)
        .execute(pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_database_schema_exists() {
    let Some(pool) = create_test_pool().await else {
        return; // Skip test if database not available
    };

    // Check that all three tables exist
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT table_name FROM information_schema.tables
         WHERE table_schema = 'public'
         AND table_name IN ('queries', 'test_definitions', 'test_results')",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query tables");

    assert_eq!(tables.len(), 3, "Expected 3 tables to exist");
    assert!(tables.contains(&"queries".to_string()));
    assert!(tables.contains(&"test_definitions".to_string()));
    assert!(tables.contains(&"test_results".to_string()));
}

#[tokio::test]
async fn test_insert_query_without_test_results() {
    let Some(pool) = create_test_pool().await else {
        return; // Skip test if database not available
    };
    let query_id = Uuid::new_v4();

    // Insert a query
    let result = sqlx::query(
        r#"
        INSERT INTO queries (
            query_id, client_ip, user_agent, numbers_sample,
            numbers_truncated, total_numbers_count, total_bits_count,
            valid, quality_score, nist_used, processing_time_ms
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(query_id)
    .bind("127.0.0.1")
    .bind("test-agent")
    .bind("0,1,2,3")
    .bind(false)
    .bind(4)
    .bind(32)
    .bind(true)
    .bind(0.75)
    .bind(true)
    .bind(100)
    .execute(&pool)
    .await;

    assert!(result.is_ok(), "Failed to insert query");

    // Verify the query was inserted
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM queries WHERE query_id = $1")
        .bind(query_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to count queries");

    assert_eq!(count, 1, "Query should be inserted");

    // Cleanup
    cleanup_test_data(&pool, query_id).await;
}

#[tokio::test]
async fn test_insert_test_definition_and_result() {
    let Some(pool) = create_test_pool().await else {
        return; // Skip test if database not available
    };
    let query_id = Uuid::new_v4();
    let test_name = format!("Test Definition {}", Uuid::new_v4());

    // First insert a query
    sqlx::query(
        r#"
        INSERT INTO queries (
            query_id, client_ip, user_agent, numbers_sample,
            numbers_truncated, total_numbers_count, total_bits_count,
            valid, quality_score, nist_used, processing_time_ms
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(query_id)
    .bind("127.0.0.1")
    .bind("test-agent")
    .bind("0,1,2,3")
    .bind(false)
    .bind(4)
    .bind(32)
    .bind(true)
    .bind(0.75)
    .bind(true)
    .bind(100)
    .execute(&pool)
    .await
    .expect("Failed to insert query");

    // Insert or get test definition
    let test_id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO test_definitions (test_name, description)
        VALUES ($1, $2)
        ON CONFLICT (test_name) DO UPDATE SET test_name = EXCLUDED.test_name
        RETURNING id
        "#,
    )
    .bind(&test_name)
    .bind("Test description")
    .fetch_one(&pool)
    .await
    .expect("Failed to insert test definition");

    // Insert test result
    let p_values_json = serde_json::json!([0.5, 0.6, 0.7]);
    let metrics_json = serde_json::json!([["metric1", "value1"], ["metric2", "value2"]]);

    let result = sqlx::query(
        r#"
        INSERT INTO test_results (query_id, test_id, passed, p_value, p_values, metrics)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(query_id)
    .bind(test_id)
    .bind(true)
    .bind(0.75)
    .bind(p_values_json)
    .bind(metrics_json)
    .execute(&pool)
    .await;

    assert!(result.is_ok(), "Failed to insert test result");

    // Verify the test result was inserted
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM test_results WHERE query_id = $1 AND test_id = $2")
            .bind(query_id)
            .bind(test_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count test results");

    assert_eq!(count, 1, "Test result should be inserted");

    // Cleanup
    cleanup_test_data(&pool, query_id).await;
}

#[tokio::test]
async fn test_foreign_key_constraint() {
    let Some(pool) = create_test_pool().await else {
        return; // Skip test if database not available
    };
    let fake_query_id = Uuid::new_v4();

    // Try to insert a test result for a non-existent query
    let result = sqlx::query(
        r#"
        INSERT INTO test_results (query_id, test_id, passed, p_value)
        VALUES ($1, 1, true, 0.5)
        "#,
    )
    .bind(fake_query_id)
    .execute(&pool)
    .await;

    // Should fail due to foreign key constraint
    assert!(
        result.is_err(),
        "Should not be able to insert test result for non-existent query"
    );
}

#[tokio::test]
async fn test_cascade_delete() {
    let Some(pool) = create_test_pool().await else {
        return; // Skip test if database not available
    };
    let query_id = Uuid::new_v4();
    let test_name = format!("Cascade Test {}", Uuid::new_v4());

    // Insert query
    sqlx::query(
        r#"
        INSERT INTO queries (
            query_id, client_ip, user_agent, numbers_sample,
            numbers_truncated, total_numbers_count, total_bits_count,
            valid, quality_score, nist_used, processing_time_ms
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(query_id)
    .bind("127.0.0.1")
    .bind("test-agent")
    .bind("0,1,2,3")
    .bind(false)
    .bind(4)
    .bind(32)
    .bind(true)
    .bind(0.75)
    .bind(true)
    .bind(100)
    .execute(&pool)
    .await
    .expect("Failed to insert query");

    // Insert test definition and result
    let test_id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO test_definitions (test_name, description)
        VALUES ($1, $2)
        ON CONFLICT (test_name) DO UPDATE SET test_name = EXCLUDED.test_name
        RETURNING id
        "#,
    )
    .bind(&test_name)
    .bind("Test description")
    .fetch_one(&pool)
    .await
    .expect("Failed to insert test definition");

    sqlx::query(
        r#"
        INSERT INTO test_results (query_id, test_id, passed, p_value)
        VALUES ($1, $2, true, 0.5)
        "#,
    )
    .bind(query_id)
    .bind(test_id)
    .execute(&pool)
    .await
    .expect("Failed to insert test result");

    // Verify test result exists
    let count_before: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM test_results WHERE query_id = $1")
            .bind(query_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count test results");
    assert_eq!(count_before, 1);

    // Delete the query
    sqlx::query("DELETE FROM queries WHERE query_id = $1")
        .bind(query_id)
        .execute(&pool)
        .await
        .expect("Failed to delete query");

    // Verify test results were cascade deleted
    let count_after: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM test_results WHERE query_id = $1")
            .bind(query_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count test results");
    assert_eq!(count_after, 0, "Test results should be cascade deleted");
}

#[tokio::test]
async fn test_unique_constraint_on_test_results() {
    let Some(pool) = create_test_pool().await else {
        return; // Skip test if database not available
    };
    let query_id = Uuid::new_v4();
    let test_name = format!("Unique Test {}", Uuid::new_v4());

    // Insert query
    sqlx::query(
        r#"
        INSERT INTO queries (
            query_id, client_ip, user_agent, numbers_sample,
            numbers_truncated, total_numbers_count, total_bits_count,
            valid, quality_score, nist_used, processing_time_ms
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(query_id)
    .bind("127.0.0.1")
    .bind("test-agent")
    .bind("0,1,2,3")
    .bind(false)
    .bind(4)
    .bind(32)
    .bind(true)
    .bind(0.75)
    .bind(true)
    .bind(100)
    .execute(&pool)
    .await
    .expect("Failed to insert query");

    // Insert test definition
    let test_id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO test_definitions (test_name, description)
        VALUES ($1, $2)
        ON CONFLICT (test_name) DO UPDATE SET test_name = EXCLUDED.test_name
        RETURNING id
        "#,
    )
    .bind(&test_name)
    .bind("Test description")
    .fetch_one(&pool)
    .await
    .expect("Failed to insert test definition");

    // Insert first test result
    sqlx::query(
        r#"
        INSERT INTO test_results (query_id, test_id, passed, p_value)
        VALUES ($1, $2, true, 0.5)
        "#,
    )
    .bind(query_id)
    .bind(test_id)
    .execute(&pool)
    .await
    .expect("Failed to insert first test result");

    // Try to insert duplicate test result for same query and test
    let result = sqlx::query(
        r#"
        INSERT INTO test_results (query_id, test_id, passed, p_value)
        VALUES ($1, $2, false, 0.3)
        "#,
    )
    .bind(query_id)
    .bind(test_id)
    .execute(&pool)
    .await;

    // Should fail due to unique constraint
    assert!(
        result.is_err(),
        "Should not be able to insert duplicate test result for same query and test"
    );

    // Cleanup
    cleanup_test_data(&pool, query_id).await;
}

#[tokio::test]
async fn test_query_test_results_join() {
    let Some(pool) = create_test_pool().await else {
        return; // Skip test if database not available
    };
    let query_id = Uuid::new_v4();

    // Insert query
    sqlx::query(
        r#"
        INSERT INTO queries (
            query_id, client_ip, user_agent, numbers_sample,
            numbers_truncated, total_numbers_count, total_bits_count,
            valid, quality_score, nist_used, processing_time_ms
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(query_id)
    .bind("127.0.0.1")
    .bind("test-agent")
    .bind("0,1,2,3")
    .bind(false)
    .bind(4)
    .bind(32)
    .bind(true)
    .bind(0.75)
    .bind(true)
    .bind(100)
    .execute(&pool)
    .await
    .expect("Failed to insert query");

    // Insert multiple test results
    for i in 1..=3 {
        let test_name = format!("Join Test {} - {}", query_id, i);
        let test_id: i32 = sqlx::query_scalar(
            r#"
            INSERT INTO test_definitions (test_name, description)
            VALUES ($1, $2)
            ON CONFLICT (test_name) DO UPDATE SET test_name = EXCLUDED.test_name
            RETURNING id
            "#,
        )
        .bind(&test_name)
        .bind("Test description")
        .fetch_one(&pool)
        .await
        .expect("Failed to insert test definition");

        sqlx::query(
            r#"
            INSERT INTO test_results (query_id, test_id, passed, p_value)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(query_id)
        .bind(test_id)
        .bind(i % 2 == 0) // Alternate pass/fail
        .bind(0.5 + (i as f64 * 0.1))
        .execute(&pool)
        .await
        .expect("Failed to insert test result");
    }

    // Query with join to get all test results for this query
    let results: Vec<(String, bool, f64)> = sqlx::query_as(
        r#"
        SELECT td.test_name, tr.passed, tr.p_value
        FROM test_results tr
        JOIN test_definitions td ON tr.test_id = td.id
        WHERE tr.query_id = $1
        ORDER BY td.test_name
        "#,
    )
    .bind(query_id)
    .fetch_all(&pool)
    .await
    .expect("Failed to query test results");

    assert_eq!(results.len(), 3, "Should have 3 test results");

    // Cleanup
    cleanup_test_data(&pool, query_id).await;
}

#[tokio::test]
async fn test_prepopulated_nist_tests() {
    let Some(pool) = create_test_pool().await else {
        return; // Skip test if database not available
    };

    // Check that NIST tests were pre-populated
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_definitions")
        .fetch_one(&pool)
        .await
        .expect("Failed to count test definitions");

    // Should have at least the 16 standard NIST tests
    assert!(
        count >= 16,
        "Should have at least 16 pre-populated NIST tests, found {}",
        count
    );

    // Check for a few specific tests
    let frequency_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM test_definitions WHERE test_name = 'Frequency (Monobit)')",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to check for Frequency test");

    assert!(frequency_exists, "Frequency test should be pre-populated");

    let runs_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM test_definitions WHERE test_name = 'Runs')")
            .fetch_one(&pool)
            .await
            .expect("Failed to check for Runs test");

    assert!(runs_exists, "Runs test should be pre-populated");
}
