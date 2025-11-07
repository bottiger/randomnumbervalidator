-- Migration: Create normalized test schema
-- This migration creates a proper normalized structure for storing individual test results

-- 1. Create queries table (replaces query_logs)
CREATE TABLE IF NOT EXISTS queries (
    id SERIAL PRIMARY KEY,
    query_id UUID NOT NULL UNIQUE,

    -- Timestamp
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- User/Client information
    client_ip VARCHAR(45),  -- IPv6 can be up to 45 chars
    user_agent TEXT,
    country VARCHAR(2),  -- ISO country code

    -- Request data
    numbers_sample TEXT NOT NULL,  -- First 5KB of random numbers
    numbers_truncated BOOLEAN NOT NULL DEFAULT FALSE,
    total_numbers_count INTEGER,
    total_bits_count INTEGER,

    -- Validation results (overall)
    valid BOOLEAN NOT NULL,
    quality_score NUMERIC(5, 4),  -- 0.0000 to 1.0000
    nist_used BOOLEAN NOT NULL DEFAULT TRUE,

    -- Processing metadata
    processing_time_ms INTEGER,
    error_message TEXT
);

-- 2. Create test_definitions table
CREATE TABLE IF NOT EXISTS test_definitions (
    id SERIAL PRIMARY KEY,
    test_name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- 3. Create test_results table
CREATE TABLE IF NOT EXISTS test_results (
    id SERIAL PRIMARY KEY,
    query_id UUID NOT NULL REFERENCES queries(query_id) ON DELETE CASCADE,
    test_id INTEGER NOT NULL REFERENCES test_definitions(id) ON DELETE CASCADE,

    -- Test results
    passed BOOLEAN NOT NULL,
    p_value NUMERIC(10, 9),  -- Primary p-value (0.000000000 to 1.000000000)
    p_values JSONB,  -- Array of p-values for tests with multiple streams
    metrics JSONB,   -- Test-specific metrics as key-value pairs

    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Ensure each query has at most one result per test
    UNIQUE(query_id, test_id)
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_queries_created_at ON queries(created_at);
CREATE INDEX IF NOT EXISTS idx_queries_client_ip ON queries(client_ip);
CREATE INDEX IF NOT EXISTS idx_queries_valid ON queries(valid);
CREATE INDEX IF NOT EXISTS idx_queries_nist_used ON queries(nist_used);

CREATE INDEX IF NOT EXISTS idx_test_results_query_id ON test_results(query_id);
CREATE INDEX IF NOT EXISTS idx_test_results_test_id ON test_results(test_id);
CREATE INDEX IF NOT EXISTS idx_test_results_passed ON test_results(passed);
CREATE INDEX IF NOT EXISTS idx_test_results_p_value ON test_results(p_value);

-- Add comments
COMMENT ON TABLE queries IS 'Stores validation queries with request details and overall results';
COMMENT ON TABLE test_definitions IS 'Defines available NIST tests and their metadata';
COMMENT ON TABLE test_results IS 'Stores individual test results for each query';

-- Prepopulate common NIST test definitions
-- These are the standard NIST SP 800-22 tests
INSERT INTO test_definitions (test_name, description) VALUES
    ('Frequency (Monobit)', 'Tests the proportion of zeros and ones in the entire sequence'),
    ('Frequency within a Block', 'Tests the proportion of ones within M-bit blocks'),
    ('Runs', 'Tests for oscillation between zeros and ones'),
    ('Longest Run of Ones in a Block', 'Tests the longest run of ones within M-bit blocks'),
    ('Binary Matrix Rank', 'Tests for linear dependence among fixed length substrings'),
    ('Discrete Fourier Transform (Spectral)', 'Tests for periodic features in the bit sequence'),
    ('Non-overlapping Template Matching', 'Tests for occurrence of pre-specified patterns'),
    ('Overlapping Template Matching', 'Tests for the number of occurrences of pre-specified patterns'),
    ('Maurer''s Universal Statistical', 'Tests whether the sequence can be compressed without loss'),
    ('Linear Complexity', 'Tests the length of a linear feedback shift register'),
    ('Serial', 'Tests the frequency of all possible overlapping m-bit patterns'),
    ('Approximate Entropy', 'Tests for the frequency of all possible overlapping m-bit patterns'),
    ('Cumulative Sums (Forward)', 'Tests the maximum excursion of the cumulative sum (forward)'),
    ('Cumulative Sums (Reverse)', 'Tests the maximum excursion of the cumulative sum (reverse)'),
    ('Random Excursions', 'Tests the number of cycles having exactly K visits in a cumulative sum'),
    ('Random Excursions Variant', 'Tests the total number of visits to a particular state')
ON CONFLICT (test_name) DO NOTHING;
