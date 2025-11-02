#!/bin/bash
set -e

echo "🔄 Random Number Validator - Update Script"
echo "=========================================="
echo ""

# Check if gcloud is installed
if ! command -v gcloud &> /dev/null; then
    echo "❌ Error: gcloud CLI is not installed."
    exit 1
fi

INSTANCE="randomvalidator-instance"
ZONE="us-central1-a"

echo "📦 Updating application code on $INSTANCE..."
echo ""

# Check if instance exists
if ! gcloud compute instances describe $INSTANCE --zone=$ZONE &> /dev/null; then
    echo "❌ Error: Instance '$INSTANCE' not found in zone '$ZONE'"
    echo "Run ./deploy.sh first to create the instance."
    exit 1
fi

echo "🔧 Pulling latest code and rebuilding..."
gcloud compute ssh $INSTANCE --zone=$ZONE --command='
    set -e
    cd /opt/randomvalidator
    echo "📥 Pulling latest changes..."
    sudo git pull
    echo "🔨 Building application..."
    source $HOME/.cargo/env
    sudo -E bash -c "source $HOME/.cargo/env && cargo build --release --bin server"
    echo "🔄 Restarting service..."
    sudo systemctl restart randomvalidator
    echo "✅ Service restarted"
    sleep 2
    sudo systemctl status randomvalidator --no-pager
'

echo ""
echo "✅ Update complete!"
echo ""
echo "To view logs:"
echo "  gcloud compute ssh $INSTANCE --zone=$ZONE --command='sudo journalctl -u randomvalidator -f'"
