#!/bin/bash
# Test NIST path resolution

echo "Testing NIST path resolution..."
echo ""

# Start server and test
echo "Starting server..."
cargo build --bin server

echo ""
echo "Testing with curl..."
sleep 1

# Start server in background
cargo run --bin server &
SERVER_PID=$!

# Wait for server to start
sleep 3

# Test the API
echo ""
echo "Sending test request..."
curl -X POST http://127.0.0.1:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{"numbers":"42,17,89,3,56,91,23,67","use_nist":true}' 2>/dev/null | jq -r '.nist_results'

# Clean up
kill $SERVER_PID 2>/dev/null

echo ""
echo "Done!"
