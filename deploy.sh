#!/bin/bash
set -e

# Deployment script for randomnumbervalidator
# This script is designed to run on the GCP instance and handle the full deployment process

REPO_DIR="/opt/randomvalidator"
LOG_FILE="/tmp/deploy-$(date +%Y%m%d-%H%M%S).log"

# Function to log messages
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

log "=== Starting deployment ==="
log "Log file: $LOG_FILE"

# Create directory if it doesn't exist
if [ ! -d "$REPO_DIR" ]; then
    log "Setting up $REPO_DIR for first time..."
    sudo mkdir -p "$REPO_DIR"
    sudo chown $USER:$USER "$REPO_DIR"
fi

# Clone or update repository
if [ ! -d "$REPO_DIR/.git" ]; then
    log "Cloning repository..."
    git clone "${REPO_URL:-https://github.com/$GITHUB_REPOSITORY.git}" "$REPO_DIR"
else
    log "Updating repository..."
    cd "$REPO_DIR"
    git fetch origin
    git reset --hard origin/main
fi

cd "$REPO_DIR"
log "Current commit: $(git rev-parse --short HEAD)"

# Build NIST test suite
log "Building NIST test suite..."
cd "$REPO_DIR/nist/sts-2.1.2/sts-2.1.2"
make clean 2>&1 | tee -a "$LOG_FILE"
make 2>&1 | tee -a "$LOG_FILE"
log "NIST test suite built successfully"

# Setup Rust if needed
cd "$REPO_DIR"
if ! command -v cargo &> /dev/null; then
    log "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 2>&1 | tee -a "$LOG_FILE"
    source "$HOME/.cargo/env"
else
    log "Rust already installed"
    source "$HOME/.cargo/env" 2>/dev/null || true
fi

# Build Rust application
log "Building Rust application (this may take several minutes)..."
cargo build --release --bin server 2>&1 | tee -a "$LOG_FILE"
log "Rust application built successfully"

# Restart service if it exists
if sudo systemctl status randomvalidator >/dev/null 2>&1; then
    log "Restarting service..."
    sudo systemctl restart randomvalidator
    log "Service restarted successfully"
else
    log "Service not configured yet - binary built successfully at target/release/server"
fi

log "=== Deployment complete ==="
log "Full logs available at: $LOG_FILE"
