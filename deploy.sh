#!/bin/bash
set -e

echo "🚀 Random Number Validator - Application Deployment"
echo "===================================================="
echo ""

# Application directory
APP_DIR="/opt/randomvalidator"
SERVICE_NAME="randomvalidator"

# Change to application directory
if [ ! -d "$APP_DIR" ]; then
    echo "❌ Error: Application directory $APP_DIR does not exist"
    echo "Run the infrastructure deployment first"
    exit 1
fi

cd "$APP_DIR"

echo "📥 Pulling latest code..."
git fetch origin
git reset --hard origin/main

echo ""
echo "🔨 Building NIST test suite..."
cd nist/sts-2.1.2/sts-2.1.2
make clean || true
make

echo ""
echo "🦀 Preparing Rust environment..."
cd "$APP_DIR"
source $HOME/.cargo/env || true

# Fix corrupted Rust toolchain
echo "  Reinstalling stable toolchain..."
rustup toolchain uninstall stable || true
rustup toolchain install stable
rustup default stable

# Verify installation
echo "  Verifying Rust installation..."
rustc --version
cargo --version

echo ""
echo "🦀 Building Rust application..."
cargo build --release --bin server

echo ""
echo "🔄 Restarting application service..."
systemctl daemon-reload
systemctl restart "$SERVICE_NAME"

echo ""
echo "⏳ Waiting for service to start..."
sleep 3

# Check service status
if systemctl is-active --quiet "$SERVICE_NAME"; then
    echo "✅ Service is running"
    echo ""
    echo "📊 Service status:"
    systemctl status "$SERVICE_NAME" --no-pager -l
else
    echo "❌ Error: Service failed to start"
    echo ""
    echo "📊 Service status:"
    systemctl status "$SERVICE_NAME" --no-pager -l
    echo ""
    echo "📋 Recent logs:"
    journalctl -u "$SERVICE_NAME" -n 50 --no-pager
    exit 1
fi

echo ""
echo "✅ Deployment complete!"
echo ""
echo "To view logs:"
echo "  sudo journalctl -u $SERVICE_NAME -f"
