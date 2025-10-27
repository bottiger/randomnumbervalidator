# Multi-stage build for optimal image size
FROM rust:1.75 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY static ./static

# Build the application
RUN cargo build --release --bin server

# Build NIST test suite
COPY nist ./nist
WORKDIR /app/nist/sts-2.1.2/sts-2.1.2
RUN make

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy built application from builder
COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /app/static /app/static
COPY --from=builder /app/nist /app/nist

# Expose port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info

# Run the server
CMD ["/app/server"]
