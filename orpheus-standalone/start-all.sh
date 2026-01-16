#!/bin/bash

# Maestro AI - Start All Services
# This script starts the complete Maestro AI platform

set -e

echo "🎵 Starting Maestro AI Platform..."
echo "=================================="
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Error: Docker is not running. Please start Docker first."
    exit 1
fi

# Start all services with Docker Compose
echo "🚀 Starting all services..."
docker-compose up -d

echo ""
echo "⏳ Waiting for services to be healthy..."
sleep 5

# Wait for services to be ready
echo "   - Waiting for PostgreSQL..."
until docker-compose exec -T postgres pg_isready -U postgres > /dev/null 2>&1; do
    sleep 1
done
echo "   ✅ PostgreSQL is ready"

echo "   - Waiting for Redis..."
until docker-compose exec -T redis redis-cli ping > /dev/null 2>&1; do
    sleep 1
done
echo "   ✅ Redis is ready"

echo "   - Waiting for MinIO..."
until curl -f http://localhost:9000/minio/health/live > /dev/null 2>&1; do
    sleep 1
done
echo "   ✅ MinIO is ready"

echo "   - Waiting for Backend API..."
until curl -f http://localhost:8080/actuator/health > /dev/null 2>&1; do
    sleep 2
done
echo "   ✅ Backend is ready"

echo ""
echo "✅ Maestro AI Platform is running!"
echo "=================================="
echo ""
echo "📱 Access Points:"
echo "   - Web Application:  http://localhost:5176"
echo "   - API Documentation: http://localhost:8080/swagger-ui"
echo "   - MinIO Console:    http://localhost:9001 (minioadmin/minioadmin)"
echo "   - Backend Health:   http://localhost:8080/actuator/health"
echo ""
echo "🎹 Available Modes:"
echo "   - Compose Mode (Cadenza AI) - Write music with tablature"
echo "   - Record Mode (Nexus DAW) - Multi-track recording"
echo "   - Mix Mode - Professional mixing console"
echo "   - Master Mode - AI-powered mastering"
echo "   - Practice Mode - Speed trainer & performance analysis"
echo "   - Distribute Mode - Multi-platform distribution"
echo ""
echo "📊 View logs with: docker-compose logs -f"
echo "🛑 Stop all services: ./stop-all.sh"
echo ""
