#!/bin/bash

# Maestro AI - Stop All Services

set -e

echo "🛑 Stopping Maestro AI Platform..."
echo "=================================="
echo ""

# Stop all services
docker-compose down

echo ""
echo "✅ All services stopped successfully"
echo ""
echo "💾 Data preserved in Docker volumes"
echo "🗑️  To remove all data, run: docker-compose down -v"
echo ""
