# 🚀 Orpheus - Quick Start Guide

Get up and running with Orpheus in 5 minutes!

---

## Prerequisites Check

Before starting, ensure you have:

- ✅ **Docker** installed and running
- ✅ **Docker Compose** installed
- ✅ At least **4GB** of free RAM
- ✅ At least **2GB** of free disk space

---

## Option 1: Docker (Recommended - Fastest)

**Perfect for: Trying out Orpheus quickly without installing dependencies**

```bash
# Start everything with Docker
./start-all.sh

# Wait for services to start (about 1-2 minutes)
# Then open your browser to http://localhost:5176
```

That's it! 🎉

### Access Points

- **🎵 Orpheus**: http://localhost:5176
- **📚 API Docs**: http://localhost:8080/swagger-ui
- **📦 MinIO Console**: http://localhost:9001

### Stop Services

```bash
./stop-all.sh
```

---

## Option 2: Local Development

**Perfect for: Development, customization, or contributing**

### Step 1: Install Prerequisites

```bash
# Check if you have everything installed
node --version   # Should be 18+
java --version   # Should be 17+
cargo --version  # Should be 1.70+
docker --version # Should be 20+
```

If any are missing:
- **Node.js**: https://nodejs.org/
- **Java**: https://adoptium.net/
- **Rust**: https://rustup.rs/
- **Docker**: https://docs.docker.com/get-docker/

### Step 2: Run Setup

```bash
# Install all dependencies
./dev-setup.sh
```

This will:
- Install Node.js dependencies
- Download Java/Kotlin dependencies
- Build the Rust audio engine

### Step 3: Build & Run

```bash
# Build all components
./build-all.sh

# Start all services
./start-all.sh
```

---

## First Steps After Starting

### 1. Open Orpheus

Go to http://localhost:5176 in your browser

### 2. Create Your First Project

Click **"New Project"** and choose:
- **Rock** - Power chords, distortion, heavy drums
- **Jazz** - Complex chords, swing feel
- **Blues** - 12-bar progression, pentatonic scales
- **Acoustic** - Fingerstyle patterns, natural sounds
- **Metal** - Drop tunings, double bass drums
- **Blank** - Start from scratch

### 3. Explore the Modes

#### 🎸 Compose Mode
- Import a Guitar Pro file (.gp3-.gp7)
- Or start writing music with the tablature editor
- Play back with MIDI

#### 🎙️ Record Mode
- Record audio or MIDI
- Layer multiple tracks
- Real-time monitoring

#### 🎚️ Mix Mode
- Balance levels
- Add EQ and compression
- Apply effects (reverb, delay)

#### 🎯 Master Mode
- Choose your target platform (Spotify, Apple Music, etc.)
- Click "Auto-Master" for AI optimization
- Export final mix

#### 🎓 Practice Mode
- Slow down difficult sections
- Loop parts for practice
- Get AI feedback

#### 🌐 Distribute Mode
- Prepare for release
- Add metadata and artwork
- Distribute to streaming platforms

---

## Quick Examples

### Import a Guitar Pro File

1. Go to **Compose Mode**
2. Click **Import** or drag-and-drop a .gp file
3. Maestro will display the tablature
4. Click **Play** to hear it

### Record a Simple Song

1. Go to **Record Mode**
2. Click **New Track** → **Audio Track**
3. Arm the track (click the **R** button)
4. Click **Record** and play your instrument
5. Click **Stop** when done

### Mix Your Track

1. Go to **Mix Mode**
2. Adjust the **volume fader** for each track
3. Use the **EQ** to shape the tone
4. Add **reverb** for space
5. Use the **AI Mixing Assistant** for suggestions

### Export Your Song

1. Go to **Master Mode**
2. Select **Export**
3. Choose format (WAV recommended for quality)
4. Click **Export** and choose location

---

## Troubleshooting

### Services won't start?

```bash
# Make sure Docker is running
docker ps

# Check if ports are free
./stop-all.sh
docker-compose down -v
./start-all.sh
```

### Can't connect to the web app?

- Make sure http://localhost:5176 is accessible
- Check backend health: http://localhost:8080/actuator/health
- View logs: `docker-compose logs -f web`

### Database errors?

```bash
# Restart database
docker-compose restart postgres

# Check logs
docker-compose logs postgres
```

### Audio service not responding?

```bash
# Restart audio service
docker-compose restart audio-service

# Check logs
docker-compose logs audio-service
```

---

## Next Steps

- 📖 Read the full [README.md](README.md) for detailed documentation
- 📚 Check the [docs/](docs/) folder for guides
- 🤖 Learn about AI personas in [docs/PERSONA_INTEGRATION.md](docs/PERSONA_INTEGRATION.md)
- 🏗️ Understand the architecture in [docs/BACKEND_ARCHITECTURE.md](docs/BACKEND_ARCHITECTURE.md)

---

## Need Help?

- Check the **Troubleshooting** section above
- Review the full documentation in README.md
- Look at container logs: `docker-compose logs -f`

---

## You're Ready! 🎉

Open http://localhost:5176 and start creating music!

**From first chord to final master - one application, infinite possibilities.** 🎵
