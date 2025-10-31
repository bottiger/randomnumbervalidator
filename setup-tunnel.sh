#!/bin/bash
# Run this script on the GCP instance to set up Cloudflare Tunnel
# Usage: ./setup-tunnel.sh <CLOUDFLARE_ACCOUNT_ID> <CLOUDFLARE_API_TOKEN>

set -e

if [ $# -ne 2 ]; then
    echo "Usage: $0 <CLOUDFLARE_ACCOUNT_ID> <CLOUDFLARE_API_TOKEN>"
    exit 1
fi

ACCOUNT_ID="$1"
API_TOKEN="$2"
TUNNEL_NAME="randomvalidator-tunnel"

echo "Setting up Cloudflare Tunnel..."

# Install cloudflared if not present
if ! command -v cloudflared &> /dev/null; then
    echo "Installing cloudflared..."
    wget -q https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64.deb
    sudo dpkg -i cloudflared-linux-amd64.deb
    rm cloudflared-linux-amd64.deb
fi

# Check if tunnel exists
TUNNELS_RESPONSE=$(curl -s -X GET "https://api.cloudflare.com/client/v4/accounts/${ACCOUNT_ID}/cfd_tunnel" \
    -H "Authorization: Bearer ${API_TOKEN}" \
    -H "Content-Type: application/json")

TUNNEL_ID=$(echo "$TUNNELS_RESPONSE" | jq -r ".result[]? | select(.name==\"${TUNNEL_NAME}\") | .id // empty")

if [ -z "$TUNNEL_ID" ]; then
    echo "Creating new tunnel..."
    RESPONSE=$(curl -s -X POST "https://api.cloudflare.com/client/v4/accounts/${ACCOUNT_ID}/cfd_tunnel" \
        -H "Authorization: Bearer ${API_TOKEN}" \
        -H "Content-Type: application/json" \
        --data "{\"name\":\"${TUNNEL_NAME}\",\"tunnel_secret\":\"$(openssl rand -base64 32)\"}")

    TUNNEL_ID=$(echo "$RESPONSE" | jq -r '.result.id // empty')
    if [ -z "$TUNNEL_ID" ]; then
        echo "Error: Failed to create tunnel"
        echo "$RESPONSE" | jq '.'
        exit 1
    fi
    echo "Created tunnel with ID: $TUNNEL_ID"
else
    echo "Using existing tunnel: $TUNNEL_ID"
fi

# Get tunnel token
TOKEN_RESPONSE=$(curl -s -X GET "https://api.cloudflare.com/client/v4/accounts/${ACCOUNT_ID}/cfd_tunnel/${TUNNEL_ID}/token" \
    -H "Authorization: Bearer ${API_TOKEN}" \
    -H "Content-Type: application/json")

TUNNEL_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.result // empty')
if [ -z "$TUNNEL_TOKEN" ]; then
    echo "Error: Failed to get tunnel token"
    echo "$TOKEN_RESPONSE" | jq '.'
    exit 1
fi

# Create config directory
sudo mkdir -p /etc/cloudflared

# Write config file
sudo tee /etc/cloudflared/config.yml > /dev/null << EOF
tunnel: ${TUNNEL_ID}
credentials-file: /etc/cloudflared/credentials.json

ingress:
  - service: http://localhost:3000
EOF

# Write tunnel token
echo "${TUNNEL_TOKEN}" | sudo tee /etc/cloudflared/credentials.json > /dev/null

# Install and start service
sudo cloudflared service install
sudo systemctl enable cloudflared
sudo systemctl restart cloudflared

echo ""
echo "✅ Cloudflare Tunnel configured successfully!"
echo ""
echo "Tunnel ID: ${TUNNEL_ID}"
echo "Tunnel hostname: ${TUNNEL_ID}.cfargotunnel.com"
echo ""
echo "Next steps:"
echo "1. Add DNS record in Cloudflare dashboard:"
echo "   Type: CNAME"
echo "   Name: @ (or randomnumbervalidator)"
echo "   Target: ${TUNNEL_ID}.cfargotunnel.com"
echo "   Proxied: Yes (orange cloud)"
echo ""
echo "2. Check tunnel status:"
echo "   sudo systemctl status cloudflared"
