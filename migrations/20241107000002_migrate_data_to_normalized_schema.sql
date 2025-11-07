-- Migration: Migrate data from query_logs to normalized schema
-- This migration moves existing data from the old query_logs table to the new structure

-- Migrate data from query_logs to queries table
INSERT INTO queries (
    query_id,
    created_at,
    client_ip,
    user_agent,
    country,
    numbers_sample,
    numbers_truncated,
    total_numbers_count,
    total_bits_count,
    valid,
    quality_score,
    nist_used,
    processing_time_ms,
    error_message
)
SELECT
    query_id,
    created_at,
    client_ip,
    user_agent,
    country,
    numbers_sample,
    numbers_truncated,
    total_numbers_count,
    total_bits_count,
    valid,
    quality_score,
    nist_used,
    processing_time_ms,
    error_message
FROM query_logs
WHERE NOT EXISTS (
    SELECT 1 FROM queries WHERE queries.query_id = query_logs.query_id
);

-- Note: Individual test results cannot be migrated from the old schema
-- because query_logs only stores summary statistics (nist_tests_passed, nist_tests_total, nist_avg_p_value)
-- The new schema will start collecting individual test results going forward

-- Optional: Drop old table after confirming migration
-- Uncomment the following line only after verifying the migration was successful
-- DROP TABLE IF EXISTS query_logs;

-- For now, we'll keep both tables to allow for rollback if needed
-- After confirming everything works, you can manually drop query_logs
