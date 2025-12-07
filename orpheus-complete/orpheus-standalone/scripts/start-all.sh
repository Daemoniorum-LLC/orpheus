#!/bin/bash
set -e

# Maestro Platform - Start All Services
# This script starts the complete platform in the correct order

echo "🎸 Starting Maestro Platform..."
echo ""

# Check if docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Navigate to project root
cd "$(dirname "$0")/.."

echo "Step 1: Starting data services (PostgreSQL, Redis, MinIO)..."
cd backend
docker-compose up -d postgres redis minio createbuckets
echo "✅ Data services started"
echo ""

# Wait for services to be healthy
echo "Waiting for services to be healthy..."
sleep 5

echo "Step 2: Starting Rust audio service..."
cd ../..
cd maestro-audio
cargo build --release --bin maestro-audio-server
nohup cargo run --release --bin maestro-audio-server > audio-service.log 2>&1 &
echo $! > audio-service.pid
echo "✅ Audio service started (PID: $(cat audio-service.pid))"
echo ""

# Wait for audio service to start
sleep 3

echo "Step 3: Starting Spring Boot backend..."
cd ../maestro/backend
nohup ./gradlew bootRun > backend.log 2>&1 &
echo $! > backend.pid
echo "✅ Backend started (PID: $(cat backend.pid))"
echo ""

# Wait for backend to start
echo "Waiting for backend to be ready..."
sleep 10

echo "Step 4: Starting web application..."
cd ../packages/app-web
npm run dev > web.log 2>&1 &
echo $! > web.pid
echo "✅ Web app started (PID: $(cat web.pid))"
echo ""

echo "═══════════════════════════════════════════════════════════"
echo "🎉 Maestro Platform is running!"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Services:"
echo "  • PostgreSQL:      http://localhost:5432"
echo "  • Redis:           http://localhost:6379"
echo "  • MinIO:           http://localhost:9001 (minioadmin/minioadmin)"
echo "  • Backend API:     http://localhost:8080"
echo "  • Swagger UI:      http://localhost:8080/swagger-ui"
echo "  • Audio Service:   grpc://localhost:50051"
echo "  • Web App:         http://localhost:5176"
echo ""
echo "Logs:"
echo "  • Audio service:   maestro-audio/audio-service.log"
echo "  • Backend:         maestro/backend/backend.log"
echo "  • Web app:         maestro/packages/app-web/web.log"
echo ""
echo "To stop all services: ./scripts/stop-all.sh"
echo ""
