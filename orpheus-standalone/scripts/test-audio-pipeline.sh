#!/bin/bash
set -e

# Maestro Audio Pipeline Test
# Tests the complete audio processing pipeline

echo "🎸 Maestro Audio Pipeline Test"
echo "═══════════════════════════════════════════════════════════"
echo ""

cd "$(dirname "$0")/../../maestro-audio"

# Check if audio service is running
if ! nc -z localhost 50051 2>/dev/null; then
    echo "❌ Audio service is not running on port 50051"
    echo "   Start it with: cargo run --release --bin maestro-audio-server"
    exit 1
fi

echo "✅ Audio service detected on port 50051"
echo ""

# Run test client
echo "Running test client..."
echo "─────────────────────────────────────────────────────────"
echo ""

cargo run --release --bin test-client

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "✅ Audio pipeline test complete!"
echo "═══════════════════════════════════════════════════════════"
