#!/bin/bash

# Maestro AI - Build All Services
# Builds frontend, backend, and audio engine

set -e

echo "🔨 Building Maestro AI Platform..."
echo "=================================="
echo ""

# Build Frontend
echo "📦 Building Frontend (React + TypeScript)..."
cd frontend
npm install
npm run build
cd ..
echo "✅ Frontend built successfully"
echo ""

# Build Backend
echo "📦 Building Backend (Spring Boot + Kotlin)..."
cd backend
./gradlew clean build -x test
cd ..
echo "✅ Backend built successfully"
echo ""

# Build Audio Engine
echo "📦 Building Audio Engine (Rust + C++)..."
cd audio-engine
cargo build --release
cd ..
echo "✅ Audio engine built successfully"
echo ""

echo "✅ All components built successfully!"
echo "=================================="
echo ""
echo "🚀 Start the platform with: ./start-all.sh"
echo ""
