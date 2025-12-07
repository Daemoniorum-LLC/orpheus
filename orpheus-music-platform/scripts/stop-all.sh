#!/bin/bash

# Maestro Platform - Stop All Services

echo "🛑 Stopping Maestro Platform..."
echo ""

cd "$(dirname "$0")/.."

# Stop web app
if [ -f packages/app-web/web.pid ]; then
    echo "Stopping web app..."
    kill $(cat packages/app-web/web.pid) 2>/dev/null || true
    rm packages/app-web/web.pid
    echo "✅ Web app stopped"
fi

# Stop backend
if [ -f backend/backend.pid ]; then
    echo "Stopping backend..."
    kill $(cat backend/backend.pid) 2>/dev/null || true
    rm backend/backend.pid
    echo "✅ Backend stopped"
fi

# Stop audio service
if [ -f ../maestro-audio/audio-service.pid ]; then
    echo "Stopping audio service..."
    kill $(cat ../maestro-audio/audio-service.pid) 2>/dev/null || true
    rm ../maestro-audio/audio-service.pid
    echo "✅ Audio service stopped"
fi

# Stop Docker services
echo "Stopping data services..."
cd backend
docker-compose down
echo "✅ Data services stopped"

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "✅ Maestro Platform stopped"
echo "═══════════════════════════════════════════════════════════"
