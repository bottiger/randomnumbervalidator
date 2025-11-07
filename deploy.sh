#!/bin/bash
set -e

echo "🚀 Random Number Validator - Application Deployment"
echo "===================================================="
echo ""

# Application directory
APP_DIR="/opt/randomvalidator"
SERVICE_NAME="randomvalidator"
DB_NAME="randomvalidator"
DB_USER="randomvalidator"
DB_PASSWORD="randomvalidator"

# Change to application directory
if [ ! -d "$APP_DIR" ]; then
    echo "❌ Error: Application directory $APP_DIR does not exist"
    echo "Run the infrastructure deployment first"
    exit 1
fi

cd "$APP_DIR"

echo "🗄️  Setting up PostgreSQL..."
# Install PostgreSQL if not already installed
if ! command -v psql &> /dev/null; then
    echo "  Installing PostgreSQL..."
    apt-get update
    apt-get install -y postgresql postgresql-contrib
fi

# Start and enable PostgreSQL service
echo "  Starting PostgreSQL service..."
systemctl start postgresql || true
systemctl enable postgresql || true

# Create database and user (idempotent - will fail silently if already exists)
echo "  Setting up database..."
sudo -u postgres psql -c "CREATE DATABASE $DB_NAME;" 2>/dev/null || echo "  Database already exists"
sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';" 2>/dev/null || echo "  User already exists"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;" 2>/dev/null || true
sudo -u postgres psql -d $DB_NAME -c "GRANT ALL ON SCHEMA public TO $DB_USER;" 2>/dev/null || true
echo "  ✅ PostgreSQL setup complete"
echo ""

echo "📥 Pulling latest code..."
git fetch origin
git reset --hard origin/main

echo ""
echo "🔨 Building NIST test suite..."
cd nist/sts-2.1.2/sts-2.1.2
make clean || true
make

echo ""
echo "🦀 Checking Rust binary..."
cd "$APP_DIR"

# Check if pre-built binary exists (from GitHub Actions)
if [ -f "$APP_DIR/target/release/server" ]; then
    echo "  ✅ Using pre-built binary from GitHub Actions"
else
    echo "  Building Rust application on server..."
    source $HOME/.cargo/env || true

    # Fix corrupted Rust toolchain if needed
    echo "  Preparing Rust environment..."
    rustup toolchain uninstall stable || true
    rustup toolchain install stable
    rustup default stable

    # Verify installation
    echo "  Verifying Rust installation..."
    rustc --version
    cargo --version

    echo "  Compiling..."
    cargo build --release --bin server
fi

echo ""
echo "⚙️  Configuring systemd service..."
cat > /etc/systemd/system/$SERVICE_NAME.service <<EOF
[Unit]
Description=Random Number Validator
After=network.target postgresql.service

[Service]
Type=simple
User=root
WorkingDirectory=$APP_DIR
Environment="RUST_LOG=info"
Environment="HOST=0.0.0.0"
Environment="PORT=3000"
Environment="DATABASE_URL=postgresql://$DB_USER:$DB_PASSWORD@localhost:5432/$DB_NAME"
ExecStart=$APP_DIR/target/release/server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF
echo "  ✅ Service configuration updated"

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
