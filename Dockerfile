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
    apt-get install -y \
        ca-certificates \
        libm6 \
        libc6 && \
    rm -rf /var/lib/apt/lists/*

# Copy built application from builder
COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /app/static /app/static
COPY --from=builder /app/nist /app/nist

# Ensure NIST assess binary is executable
RUN chmod +x /app/nist/sts-2.1.2/sts-2.1.2/assess

# Create necessary directories for NIST tests
RUN mkdir -p /app/nist/sts-2.1.2/sts-2.1.2/data && \
    mkdir -p /app/nist/sts-2.1.2/sts-2.1.2/experiments

# Expose port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD timeout 2 bash -c "</dev/tcp/localhost/3000" || exit 1

# Run the server
CMD ["/app/server"]
