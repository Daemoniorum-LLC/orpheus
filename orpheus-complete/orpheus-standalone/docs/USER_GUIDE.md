# 🎸 Orpheus - Complete User Guide

> **From first chord to final master - one application, infinite possibilities.**

Welcome to Orpheus, the world's first unified music production platform. This guide will help you master all aspects of the application.

---

## 📋 Table of Contents

- [Getting Started](#getting-started)
- [Compose Mode](#compose-mode)
- [Record Mode](#record-mode)
- [Mix Mode](#mix-mode)
- [Master Mode](#master-mode)
- [Practice Mode](#practice-mode)
- [Distribute Mode](#distribute-mode)
- [AI Assistant](#ai-assistant)
- [Keyboard Shortcuts](#keyboard-shortcuts)
- [Tips & Best Practices](#tips--best-practices)
- [Troubleshooting](#troubleshooting)

---

## 🚀 Getting Started

### First Launch

1. **Grant Permissions**: On first use, Maestro will request microphone access for recording features. Click "Allow" to enable full functionality.

2. **Start a New Project or Import**:

   **Option A: Create New Project**
   - Click the **New Project** button in Compose Mode
   - Choose from 6 professional templates (Rock Band, Blues Trio, Jazz Quartet, etc.)
   - Enter project title and artist name (optional)
   - Start composing immediately with pre-configured tracks

   **Option B: Import Existing File**
   - Click the **Open** button in the toolbar
   - Select a Guitar Pro file (`.gp3` - `.gp7`) or `.maestro` project
   - Or simply **drag and drop** any file directly into the application
   - The tablature will automatically load in Compose Mode

3. **Explore the Interface**:
   - **Toolbar** (top): File operations, playback controls, undo/redo, AI assistant
   - **Mode Selector** (below toolbar): Switch between 6 production modes
   - **Sidebar** (left): Project browser and file management
   - **Main Area** (center): Mode-specific workspace
   - **AI Panel** (right): Context-aware assistant (toggle on/off)

### Auto-Save and Recovery

Orpheus automatically saves your work to prevent data loss:

- **Auto-Save**: Your project is saved to browser storage every 30 seconds while you work
- **Auto-Recovery**: If the browser crashes or you close the tab, Maestro will offer to recover your work when you return (within 1 hour)
- **Manual Save**: Press `Ctrl+S` at any time to download a `.maestro` file to your computer

**Recovery Workflow:**
1. Reopen Maestro after unexpected close
2. If auto-save exists (< 1 hour old), you'll see a recovery prompt
3. Click "Recover" to restore your work, or "Discard" to start fresh

---

## 🎼 Compose Mode

**Purpose:** Tablature and notation editing with Guitar Pro import

### Features

- **Project Templates**: 6 professional templates (Blank, Rock Band, Blues Trio, Jazz Quartet, Acoustic Duo, Metal)
- **Native Tablature Rendering**: Full Guitar Pro (GP3-GP7) support with all techniques preserved
- **MIDI Playback**: Realistic guitar synthesis with synchronized cursor
- **Timeline Control**: Adjust tempo, loop sections, jump to measures
- **Chord Library**: 100+ chord types with fingering diagrams
- **Scale Library**: 20+ scales for improvisation and composition
- **Undo/Redo**: 50-state history to revert changes (Ctrl+Z / Ctrl+Shift+Z)

### Workflow

1. **Create a New Project**
   ```
   Click "New Project" → Choose template → Enter title/artist → Start composing
   ```

   **Available Templates:**
   - **Blank Project**: Single guitar track (C major, 120 BPM)
   - **Rock Band**: Lead guitar, rhythm guitar, bass, drums (E major, 140 BPM)
   - **Blues Trio**: Guitar, bass, drums (A major, 120 BPM)
   - **Jazz Quartet**: Piano, guitar, bass, drums (Bb major, 160 BPM)
   - **Acoustic Duo**: Two acoustic guitars (G major, 90 BPM)
   - **Metal Band**: Lead/rhythm guitars, bass, drums, Drop D (D major, 180 BPM)

2. **Import Guitar Pro File**
   ```
   Click "Open" → Select .gp file → Tablature loads automatically
   OR
   Drag and drop .gp file anywhere in the app
   ```

3. **Playback**
   - Press **Space** to play/pause
   - Adjust **Tempo** slider for practice speeds
   - Use **Timeline** to jump to specific measures
   - Watch synchronized cursor highlight current position

4. **Make Edits**
   - Edit notes, chords, and measures
   - Use **Ctrl+Z** to undo changes
   - Use **Ctrl+Shift+Z** to redo changes
   - Press **Ctrl+S** to save your work

5. **AI Composition Help**
   - Click **AI Assistant** in toolbar
   - Ask Music Theory Tutor about:
     - Chord progressions
     - Scale selection
     - Harmony and melody
     - Song structure

### Example Questions for AI

> "What chords work well in the key of D major?"

> "How do I write a chorus that contrasts with my verse?"

> "What scale should I use for a blues solo in A?"

---

## 🎙️ Record Mode

**Purpose:** Multi-track audio recording with real-time monitoring

### Features

- **Professional Recording**: 48kHz sampling with echo cancellation disabled
- **Live Monitoring**: Real-time waveform and spectrum visualization
- **Pause/Resume**: Non-destructive recording control
- **Multi-Track**: Record multiple takes and manage tracks independently
- **Export**: Download tracks as WebM files

### Recording Workflow

1. **Setup**
   - Ensure microphone is connected and working
   - Grant microphone permission if prompted
   - Watch live visualizer to confirm input level

2. **Record a Track**
   ```
   1. Click "Record" button (turns red)
   2. Perform your take
   3. Use "Pause" if needed (optional)
   4. Click "Stop" when finished
   5. Enter track name (or leave default)
   ```

3. **Manage Tracks**
   - View all recordings in track list
   - **Export** individual tracks as WebM files
   - **Delete** unwanted takes
   - Recordings show duration and timestamp

### Recording Tips

- **Input Gain**: Aim for peaks around -12dB (avoid clipping)
- **Room Treatment**: Record in a quiet space with minimal reverb
- **Microphone Placement**: 
  - Acoustic guitar: 6-12" from 12th fret, angled toward soundhole
  - Electric guitar amp: 1-2" from speaker cone, slightly off-center
  - Vocals: 6-8" from mouth, use pop filter

### AI Session Assistance

Ask the Session Assistant about:
- Microphone techniques
- Input gain staging
- Recording environment setup
- Performance coaching

> "How do I reduce background noise in my recordings?"

> "What's the best way to mic a guitar amp?"

---

## 🎚️ Mix Mode

**Purpose:** Professional mixing console with AI suggestions

### Features

- **Channel Strips**: Professional mixer layout with:
  - Dual VU meters (60fps real-time animation)
  - Volume faders (-60dB to +12dB)
  - Pan controls (-50L to +50R center)
  - Solo/Mute buttons
  - Delete track option

- **Master Channel**: Dedicated master fader with separate pan control

- **Dynamic Track Management**: Add/remove tracks on the fly

- **AI Mix Suggestions**: Intelligent mixing guidance via Mixing Engineer persona

### Mixing Workflow

1. **Load Tracks**
   - Default tracks appear: Guitar, Bass, Drums, Vocals
   - Click **Add Track** to create more channels
   - Delete unwanted tracks with trash icon

2. **Balance Levels**
   ```
   1. Set all faders to 0dB initially
   2. Pull down each track to -∞ (mute)
   3. Bring up kick drum first
   4. Add bass next
   5. Layer in other instruments one at a time
   6. Adjust relative levels until balanced
   ```

3. **Panning**
   - Keep bass, kick, snare, vocals **centered** (Pan 0)
   - Pan guitars **left and right** for width (-30L, +30R)
   - Pan hi-hats slightly off-center for interest

4. **Use Solo/Mute**
   - **Solo**: Isolate one track to focus on it
   - **Mute**: Temporarily remove track from mix
   - Compare with/without effects to judge impact

5. **Master Fader**
   - Adjust overall mix volume
   - Keep master peaks below -6dB for headroom
   - Leave room for mastering stage

### Mixing Best Practices

- **Start with Levels**: Get balance right before adding effects
- **Reference Mixes**: Compare to professional tracks in same genre
- **Take Breaks**: Ear fatigue affects judgment - rest every hour
- **Check Mono**: Ensure mix translates to mono playback
- **Headroom**: Leave 6dB of headroom on master for mastering

### AI Mixing Guidance

Ask the Mixing Engineer about:
- EQ and frequency balance
- Compression techniques
- Spatial effects (reverb, delay)
- Mix troubleshooting

> "My guitar and bass sound muddy together - how do I fix it?"

> "What compression settings should I use for vocals?"

---

## ✨ Master Mode

**Purpose:** AI-powered mastering with platform-specific targets

### Features

- **ITU-R BS.1770-4 Loudness Metering**:
  - **Integrated LUFS**: Overall loudness (target: -14 LUFS)
  - **True Peak**: Peak level (keep below -1dBTP)
  - **Dynamic Range**: Punch and musicality (aim for 8-12dB)

- **Platform-Specific Targets**:
  - Spotify: -14 LUFS
  - Apple Music: -16 LUFS
  - YouTube: -14 LUFS
  - Tidal: -14 LUFS
  - SoundCloud: -8 to -13 LUFS

- **Multi-Format Export**:
  - WAV (44.1kHz/24-bit, 48kHz/24-bit)
  - FLAC (Lossless compression)
  - MP3 (320kbps high quality)
  - AAC (256kbps Apple-optimized)

### Mastering Workflow

1. **Select Target Platform**
   ```
   Choose from dropdown: Spotify, Apple Music, YouTube, etc.
   Target LUFS displayed for reference
   ```

2. **Monitor Loudness**
   - Watch **LUFS-I** meter in real-time
   - Ensure **True Peak** stays below -1dBTP
   - Check **Dynamic Range** stays between 8-12dB

3. **Optimize for Target**
   - If LUFS too low: Add limiting or compression
   - If LUFS too high: Reduce overall gain
   - Avoid over-compression (preserve dynamics!)

4. **Export Master**
   ```
   Select format: WAV 44.1kHz/24-bit (recommended)
   Click Export button
   Download master file
   ```

### Mastering Standards

| Platform | Target LUFS | True Peak | Notes |
|----------|-------------|-----------|-------|
| Spotify | -14 LUFS | -1 dBTP | Normalizes automatically |
| Apple Music | -16 LUFS | -1 dBTP | Slightly quieter target |
| YouTube | -14 LUFS | -1 dBTP | Same as Spotify |
| Tidal | -14 LUFS | -1 dBTP | HiFi streaming |
| SoundCloud | -8 to -13 LUFS | -1 dBTP | Prefers louder |
| CD | -9 to -11 LUFS | -0.3 dBTP | Traditional mastering |
| Vinyl | -14 to -16 LUFS | -3 dBTP | Needs extra headroom |

### AI Mastering Assistance

Click **AI Auto-Master** to ask the Mastering Engineer:

> "What's the difference between -14 LUFS and -16 LUFS?"

> "My master sounds distorted - what went wrong?"

> "How do I master for both streaming and CD?"

---

## 🎸 Practice Mode

**Purpose:** Speed trainer with AI performance analysis

### Features

- Loop section playback
- Variable speed (25% - 200%)
- Metronome with accents
- Practice session tracking
- AI Guitar Coach guidance

### Practice Workflow

1. **Select Passage**
   - Load Guitar Pro file in Compose Mode
   - Switch to Practice Mode
   - Select measures to loop

2. **Set Initial Tempo**
   ```
   Start at 50-60% of target speed
   Perfect the motions slowly
   Gradually increase by 5 BPM
   ```

3. **Practice Session**
   - Loop section repeatedly
   - Focus on accuracy over speed
   - Use metronome for timing
   - Rest hands between reps

### Speed Building Tips

- **Slow is Fast**: Master technique slowly before adding speed
- **Incremental Progress**: Only increase 5 BPM when you nail it 3 times
- **Burst Practice**: Short 5-10 min sessions with breaks
- **Relaxation**: Tension is the enemy - stay loose
- **Accuracy First**: Speed comes naturally with perfect technique

### AI Practice Coaching

> "How do I build alternate picking speed?"

> "What's a good warm-up routine for guitar?"

> "How do I overcome tension when playing fast passages?"

---

## 🌍 Distribute Mode

**Purpose:** Music distribution to streaming platforms

### Features

- **Release Metadata Forms**
- **Artwork Upload** (3000x3000px)
- **Platform Selection** (Spotify, Apple Music, etc.)
- **Distribution Checklist**
- **DistroKid Integration**
- **Release Scheduling**

### Distribution Workflow

1. **Prepare Audio**
   - Export final master from Master Mode
   - Ensure WAV 44.1kHz/24-bit or higher
   - Verify LUFS target for your chosen platform

2. **Fill in Metadata**
   ```
   Track Title: Enter song name
   Artist Name: Your artist name (exactly as desired)
   Album Title: Single, EP, or Album name
   Genre: Select primary genre
   ISRC: Auto-generate or enter your own
   UPC: Auto-generate or enter barcode
   Lyrics: Optional (for platforms that display)
   Explicit: Check if contains explicit content
   ```

3. **Upload Artwork**
   - Click artwork upload area
   - Select 3000x3000px JPG or PNG
   - Must be square (1:1 aspect ratio)
   - RGB color mode
   - No contact info, URLs, or misleading content

4. **Select Platforms**
   ```
   ✅ Spotify (1-2 days delivery)
   ✅ Apple Music (1-2 days)
   ✅ YouTube Music (1-2 days)
   ✅ Amazon Music (1-2 days)
   ⬜ Tidal (3-5 days)
   ⬜ Deezer (3-5 days)
   ```

5. **Schedule Release**
   - Set release date (minimum 2 weeks out for editorial review)
   - Goes live at midnight in each timezone
   - Earlier submission = better playlist consideration

6. **Submit**
   - Review distribution checklist
   - Click "Submit to X Platforms"
   - Monitor upload progress

### Artwork Requirements

✅ **Required:**
- Minimum 3000x3000 pixels (recommended)
- JPG or PNG format
- Perfect square (1:1 aspect ratio)
- RGB color mode

⚠️ **Prohibited:**
- Contact information (phone, email, URLs)
- Explicit or graphic content (unless marked explicit)
- Misleading imagery
- Copyright infringement

### Distribution Best Practices

- **ISRC Codes**: Use them to track your music across platforms
- **Release Timing**: Submit 2-4 weeks early for playlist consideration
- **Metadata Accuracy**: Spelling errors can't be fixed after release
- **Genre Selection**: Choose carefully - affects algorithmic playlisting
- **Royalties**: With DistroKid, you keep 100% of earnings

### AI Distribution Guidance

> "What's an ISRC code and do I need one?"

> "How long before release should I submit to platforms?"

> "What are the requirements for Spotify playlist consideration?"

---

## 🤖 AI Assistant

Orpheus features **6 specialized AI personas**, each expertly trained for their mode.

### How to Use

1. **Open AI Panel**: Click AI icon in toolbar or mode-specific AI button
2. **Persona Auto-Switches**: Changes based on current mode
3. **Ask Questions**: Type in chat box, press Enter to send
4. **Get Guidance**: Receive context-aware, mode-specific help
5. **Clear Chat**: Click trash icon to start fresh conversation

### Persona Guide

| Mode | Persona | Expertise |
|------|---------|-----------|
| Compose | 🎼 Music Theory Tutor | Chords, scales, harmony, melody, structure |
| Record | 🎙️ Session Assistant | Mics, gain staging, recording techniques |
| Mix | 🎚️ Mixing Engineer | EQ, compression, effects, balance |
| Master | ✨ Mastering Engineer | LUFS, platform targets, mastering chains |
| Practice | 🎸 Guitar Coach | Technique, speed building, practice routines |
| Distribute | 🌍 Distribution Manager | Platforms, metadata, release strategies |

### Example Conversations

**Compose Mode:**
> User: "I need a chord progression for a sad ballad in Am"
> 
> AI: "For a sad ballad in A minor, try this progression:
> 
> • Am - F - C - G (very emotional, descending feel)
> • Am - Dm - E - Am (classic minor, strong resolution)
> • Am - G - F - E (dramatic, rising tension)
> 
> Start with the first one - it's used in countless ballads. Which instruments are you writing for?"

**Record Mode:**
> User: "How do I mic an acoustic guitar?"
> 
> AI: "For recording acoustic guitar, here's what I recommend:
> 
> • Place the mic 6-12 inches from the 12th fret
> • Angle it slightly toward the soundhole
> • Use a cardioid pattern microphone
> • Set input gain so peaks hit around -12dB
> 
> Are you recording acoustic or electric?"

### Tips for Better AI Responses

- **Be Specific**: "How do I EQ muddy bass?" vs "How do I mix?"
- **Provide Context**: "I'm mastering for Spotify - what LUFS target?"
- **Ask Follow-ups**: Continue conversation for deeper guidance
- **Use Mode-Specific Language**: The AI understands technical terms

---

## ⌨️ Keyboard Shortcuts

### Global

| Shortcut | Action |
|----------|--------|
| `Space` | Play/Pause |
| `Home` | Jump to start |
| `End` | Jump to end |
| `Ctrl + O` | Open project |
| `Ctrl + S` | Save project (downloads .maestro file) |
| `Ctrl + Z` | Undo (50-state history) |
| `Ctrl + Shift + Z` | Redo |

### Mode Switching

| Shortcut | Mode |
|----------|------|
| `Ctrl + 1` | Compose Mode |
| `Ctrl + 2` | Record Mode |
| `Ctrl + 3` | Mix Mode |
| `Ctrl + 4` | Master Mode |
| `Ctrl + 5` | Practice Mode |
| `Ctrl + 6` | Distribute Mode |

### Playback

| Shortcut | Action |
|----------|--------|
| `Space` | Play/Pause |
| `J` | Rewind |
| `K` | Stop |
| `L` | Fast Forward |
| `,` | Previous measure |
| `.` | Next measure |

### Recording

| Shortcut | Action |
|----------|--------|
| `R` | Start/Stop Recording |
| `P` | Pause/Resume |

---

## 💡 Tips & Best Practices

### General Workflow

1. **Compose First**: Start with solid composition in Compose Mode
2. **Record Clean**: Capture best possible performance in Record Mode
3. **Mix Thoughtfully**: Balance and enhance in Mix Mode
4. **Master Carefully**: Prepare for distribution in Master Mode
5. **Distribute Strategically**: Release with proper metadata in Distribute Mode

### Audio Quality

- **Record at Highest Quality**: 48kHz/24-bit or higher
- **Preserve Headroom**: Keep peaks below -6dB during recording/mixing
- **Export Lossless**: Use WAV or FLAC for masters
- **Compress Last**: Only convert to MP3/AAC for final distribution

### Performance Optimization

- **Close Unused Modes**: Free up resources by staying in one mode
- **Export Regularly**: Save incremental versions of your work
- **Restart Browser**: Clear memory if performance degrades

### Creative Workflow

- **Reference Tracks**: Compare your mix to professional releases
- **Take Breaks**: Fresh ears make better decisions
- **Version Control**: Save multiple mix/master versions
- **Get Feedback**: Share work-in-progress with trusted listeners

---

## 🔧 Troubleshooting

### Audio Issues

**Problem:** No sound during playback
- ✅ Check browser audio permissions
- ✅ Verify system volume is up
- ✅ Try different browser (Chrome recommended)
- ✅ Reload page and re-import file

**Problem:** Microphone not detected
- ✅ Grant microphone permission when prompted
- ✅ Check browser settings > Privacy > Microphone
- ✅ Ensure microphone is connected and working in system
- ✅ Try different browser

**Problem:** Crackling/distortion in audio
- ✅ Reduce input gain (aim for -12dB peaks)
- ✅ Close other tabs/applications
- ✅ Increase browser audio buffer size
- ✅ Update audio drivers

### File Import Issues

**Problem:** Guitar Pro file won't import
- ✅ Verify file format (GP3-GP7 supported)
- ✅ Check file isn't corrupted (open in Guitar Pro first)
- ✅ Try exporting from Guitar Pro in older format (GP5)
- ✅ Contact support with file details

**Problem:** Tablature renders incorrectly
- ✅ Refresh page and re-import
- ✅ Check source file in Guitar Pro
- ✅ Some advanced techniques may have limited support

**Problem:** Drag and drop not working
- ✅ Ensure you're dragging a supported file type (.gp3-7, .maestro, .maestro.json)
- ✅ Try using the Open button instead
- ✅ Check browser console for errors
- ✅ Refresh page and try again

### Save/Recovery Issues

**Problem:** Auto-save not working
- ✅ Check browser localStorage is enabled
- ✅ Clear browser storage if full (may lose auto-save)
- ✅ Use manual save (Ctrl+S) as backup
- ✅ Some browsers in private mode disable localStorage

**Problem:** Auto-recovery prompt doesn't appear
- ✅ Auto-save only kept for 1 hour
- ✅ Check if you manually saved before closing
- ✅ Browser storage may have been cleared
- ✅ Try checking browser console for errors

**Problem:** Undo/Redo not working
- ✅ History limited to 50 states (older changes discarded)
- ✅ Undo only works on edits made with updateProject()
- ✅ Cannot undo past initial file load
- ✅ Check if you're typing in an input field (shortcuts disabled during text input)

### Performance Issues

**Problem:** UI is laggy/slow
- ✅ Close unnecessary browser tabs
- ✅ Disable browser extensions
- ✅ Clear browser cache
- ✅ Use Chrome or Firefox (best performance)
- ✅ Reload page

**Problem:** Playback stutters
- ✅ Reduce zoom level in Compose Mode
- ✅ Close AI Assistant panel if open
- ✅ Simplify project (fewer tracks)
- ✅ Increase audio buffer size

### AI Assistant Issues

**Problem:** AI not responding
- ✅ Check internet connection
- ✅ Reload page
- ✅ Clear chat and try again
- ✅ API may be temporarily unavailable

**Problem:** AI gives incorrect information
- ✅ AI responses are simulated in current version
- ✅ Full AI integration coming in future update
- ✅ Cross-reference with trusted sources

---

## 🎓 Learning Resources

### Built-in Help

- **AI Assistant**: Ask questions anytime in any mode
- **Tooltips**: Hover over controls for quick info
- **Empty States**: Each mode explains its purpose when first opened

### External Resources

**Music Theory:**
- musictheory.net
- teoria.com
- justinguitar.com

**Recording:**
- Sound On Sound magazine
- Produce Like A Pro (YouTube)
- Recording Revolution (YouTube)

**Mixing:**
- Mix With The Masters
- Pensado's Place
- Audio Engineering Society (AES)

**Mastering:**
- Ian Shepherd Mastering
- iZotope Mastering Guide
- Sage Audio

**Distribution:**
- DistroKid Help Center
- CD Baby DIY Musician Podcast
- Spotify for Artists Blog

---

## 📧 Support

**Need Help?**
- Check this User Guide first
- Ask the AI Assistant in the app
- Visit documentation: https://maestro-ai.dev/docs
- Contact support: support@maestro-ai.dev

**Found a Bug?**
- Report on GitHub: https://github.com/yourusername/maestro-ai/issues
- Include browser info, steps to reproduce, and screenshots

**Feature Requests:**
- Submit on GitHub Discussions
- Vote on existing requests
- Contribute to open source development

---

<div align="center">

**Happy Music Making! 🎵**

Built with ❤️ by musicians, for musicians.

</div>
