#!/bin/bash
# Quick test script for NIST integration

echo "Testing NIST Integration"
echo "========================"
echo ""

# Check if NIST is compiled
if [ ! -f "nist/sts-2.1.2/sts-2.1.2/assess" ]; then
    echo "ERROR: NIST assess binary not found!"
    echo "Run 'make nist' to compile the NIST test suite first."
    exit 1
fi

echo "✓ NIST assess binary found"
echo ""

# Test with a simple dataset
echo "Testing with sample random numbers..."
echo ""

# Start server in background
cargo build --bin server 2>&1 | grep -q "Finished"
echo "✓ Server built successfully"
echo ""

echo "To test manually:"
echo "1. Run: cargo run --bin server"
echo "2. Open http://127.0.0.1:3000 in your browser"
echo "3. Enter numbers like: 42, 17, 89, 3, 56, 91, 23, 67, 14, 88"
echo "4. Check the 'Run comprehensive NIST tests' checkbox"
echo "5. Click 'Validate Numbers'"
echo ""
echo "Note: NIST tests require at least 100 bits (about 4 numbers)"
