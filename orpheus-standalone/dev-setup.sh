#!/bin/bash

# Maestro AI - Development Setup
# Initial setup for development environment

set -e

echo "🔧 Maestro AI - Development Setup"
echo "=================================="
echo ""

# Check prerequisites
echo "Checking prerequisites..."
echo ""

# Check Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js 18+ first."
    exit 1
fi
echo "✅ Node.js $(node --version)"

# Check npm
if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed. Please install npm first."
    exit 1
fi
echo "✅ npm $(npm --version)"

# Check Java
if ! command -v java &> /dev/null; then
    echo "❌ Java is not installed. Please install Java 17+ first."
    exit 1
fi
echo "✅ Java $(java --version | head -n 1)"

# Check Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi
echo "✅ Rust $(rustc --version)"

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi
echo "✅ Docker $(docker --version)"

# Check Docker Compose
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi
echo "✅ Docker Compose $(docker-compose --version)"

echo ""
echo "✅ All prerequisites met!"
echo ""

# Install frontend dependencies
echo "📦 Installing frontend dependencies..."
cd frontend
npm install
cd ..
echo "✅ Frontend dependencies installed"
echo ""

# Download Gradle dependencies (backend)
echo "📦 Downloading backend dependencies..."
cd backend
./gradlew dependencies
cd ..
echo "✅ Backend dependencies downloaded"
echo ""

# Build audio engine
echo "📦 Building audio engine..."
cd audio-engine
cargo build
cd ..
echo "✅ Audio engine built"
echo ""

echo "✅ Development setup complete!"
echo "=================================="
echo ""
echo "🚀 Next steps:"
echo "   1. Build all components: ./build-all.sh"
echo "   2. Start the platform: ./start-all.sh"
echo "   3. Open http://localhost:5176"
echo ""
