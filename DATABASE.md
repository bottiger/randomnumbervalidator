# Database Query Logging

This application supports optional database logging of all validation queries. When enabled, it logs detailed information about each request for analytics and monitoring purposes.

## Features

The database logging captures:

- **Timestamp**: When the query was made (with timezone)
- **Request data**: First 5KB of random numbers submitted
- **Truncation flag**: Whether the logged numbers were truncated
- **User information**: Client IP, User-Agent, and country (for geolocation)
- **Query metrics**: Total number count, total bits analyzed
- **Validation results**: Valid/invalid status, quality score
- **NIST metrics**: Tests passed/total, average p-value
- **Performance**: Processing time in milliseconds
- **Unique ID**: UUID for each query

## Setup

### 1. PostgreSQL Database

You'll need a PostgreSQL database. You can set one up locally or use a cloud provider.

**Local PostgreSQL with Docker:**

```bash
docker run -d \
  --name randomvalidator-db \
  -e POSTGRES_PASSWORD=yourpassword \
  -e POSTGRES_DB=randomvalidator \
  -p 5432:5432 \
  postgres:16
```

**Or use docker-compose (add to docker-compose.yml):**

```yaml
services:
  db:
    image: postgres:16
    environment:
      POSTGRES_PASSWORD: yourpassword
      POSTGRES_DB: randomvalidator
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

### 2. Set DATABASE_URL Environment Variable

```bash
export DATABASE_URL="postgresql://postgres:yourpassword@localhost:5432/randomvalidator"
```

Or add it to a `.env` file:

```
DATABASE_URL=postgresql://postgres:yourpassword@localhost:5432/randomvalidator
```

### 3. Run the Application

When you start the server with `DATABASE_URL` set, it will:

1. Connect to the database
2. Automatically run migrations to create the `query_logs` table
3. Start logging all validation requests

```bash
cargo run --bin server
```

You should see:
```
Database connection established
Database migrations completed
Database logging enabled
```

## Database Schema

The `query_logs` table structure:

```sql
CREATE TABLE query_logs (
    id SERIAL PRIMARY KEY,
    query_id UUID NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- User/Client information
    client_ip VARCHAR(45),
    user_agent TEXT,
    country VARCHAR(2),

    -- Request data
    numbers_sample TEXT NOT NULL,
    numbers_truncated BOOLEAN NOT NULL DEFAULT FALSE,
    total_numbers_count INTEGER,
    total_bits_count INTEGER,

    -- Validation results
    valid BOOLEAN NOT NULL,
    quality_score NUMERIC(5, 4),
    nist_used BOOLEAN NOT NULL DEFAULT TRUE,

    -- NIST metrics
    nist_tests_passed INTEGER,
    nist_tests_total INTEGER,
    nist_avg_p_value NUMERIC(5, 4),

    -- Performance
    processing_time_ms INTEGER,
    error_message TEXT
);
```

## Usage Without Database

The application works perfectly fine without a database! If `DATABASE_URL` is not set:

- The server will start normally
- All validation functionality works as expected
- Queries are logged to stdout/stderr only
- You'll see: `Database logging disabled (set DATABASE_URL to enable)`

This makes development and testing easier without requiring a database setup.

## Querying the Database

### View recent queries:

```sql
SELECT
    created_at,
    client_ip,
    total_numbers_count,
    valid,
    quality_score,
    processing_time_ms
FROM query_logs
ORDER BY created_at DESC
LIMIT 10;
```

### Get statistics:

```sql
SELECT
    COUNT(*) as total_queries,
    COUNT(*) FILTER (WHERE valid) as valid_queries,
    AVG(quality_score) as avg_quality_score,
    AVG(processing_time_ms) as avg_processing_time
FROM query_logs
WHERE created_at > NOW() - INTERVAL '24 hours';
```

### Top clients by query count:

```sql
SELECT
    client_ip,
    COUNT(*) as query_count,
    AVG(quality_score) as avg_quality
FROM query_logs
WHERE created_at > NOW() - INTERVAL '7 days'
GROUP BY client_ip
ORDER BY query_count DESC
LIMIT 10;
```

### Find failed validations:

```sql
SELECT
    query_id,
    created_at,
    client_ip,
    quality_score,
    numbers_sample
FROM query_logs
WHERE valid = false
ORDER BY created_at DESC
LIMIT 20;
```

## IP Geolocation (Optional)

The `country` field is currently NULL by default. To populate it, you can:

1. Use a background job to lookup IPs and update the field
2. Integrate a geolocation service (like MaxMind GeoIP2)
3. Use a trigger/function in PostgreSQL

Example with MaxMind GeoLite2:

```sql
-- Add a trigger to populate country on insert
-- (Requires pgGeoIP extension or external service)
```

This is left as an optional enhancement for production deployments.

## Production Recommendations

1. **Use connection pooling**: The app already uses SQLx connection pooling (max 5 connections)
2. **Add indexes**: The migration includes indexes on `created_at`, `client_ip`, and `valid`
3. **Set up backups**: Regular PostgreSQL backups
4. **Monitor disk usage**: Consider archiving old logs
5. **Use read replicas**: For analytics queries to avoid impacting the main database

## Environment Variables

- `DATABASE_URL`: PostgreSQL connection string (optional)
- `HOST`: Server bind address (default: 0.0.0.0)
- `PORT`: Server port (default: 3000)
- `RUST_LOG`: Logging level (default: randomnumbervalidator=info)

## Troubleshooting

**"Failed to connect to database"**
- Verify `DATABASE_URL` is correct
- Check PostgreSQL is running
- Ensure network connectivity

**"Failed to run migrations"**
- Check database permissions
- Verify the migrations directory exists
- Try running migrations manually with `sqlx migrate run`

**Queries not being logged**
- Check server logs for errors
- Verify database connection was established at startup
- Check database permissions for INSERT operations
