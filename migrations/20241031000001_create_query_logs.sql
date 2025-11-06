-- Create query_logs table for logging all validation requests
CREATE TABLE IF NOT EXISTS query_logs (
    id SERIAL PRIMARY KEY,
    query_id UUID NOT NULL UNIQUE,

    -- Timestamp
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- User/Client information
    client_ip VARCHAR(45),  -- IPv6 can be up to 45 chars
    user_agent TEXT,
    country VARCHAR(2),  -- ISO country code (can be populated later with geolocation)

    -- Request data
    numbers_sample TEXT NOT NULL,  -- First 5KB of random numbers
    numbers_truncated BOOLEAN NOT NULL DEFAULT FALSE,  -- Whether sample was truncated
    total_numbers_count INTEGER,  -- Total count of numbers submitted
    total_bits_count INTEGER,      -- Total bits analyzed

    -- Validation results (summary)
    valid BOOLEAN NOT NULL,
    quality_score NUMERIC(5, 4),  -- 0.0000 to 1.0000
    nist_used BOOLEAN NOT NULL DEFAULT TRUE,

    -- Key metrics summary
    nist_tests_passed INTEGER,
    nist_tests_total INTEGER,
    nist_avg_p_value NUMERIC(5, 4),

    -- Additional metadata
    processing_time_ms INTEGER,  -- How long the validation took
    error_message TEXT            -- If there was an error
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_created_at ON query_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_client_ip ON query_logs(client_ip);
CREATE INDEX IF NOT EXISTS idx_valid ON query_logs(valid);

-- Add comment to table
COMMENT ON TABLE query_logs IS 'Logs all random number validation queries with request details and results';
