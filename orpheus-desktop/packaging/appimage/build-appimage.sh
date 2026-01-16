#!/bin/bash
# Build Orpheus AppImage
# Requires: appimagetool, cargo

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target/appimage"
APPDIR="$BUILD_DIR/Orpheus.AppDir"

echo "=== Building Orpheus AppImage ==="
echo "Project root: $PROJECT_ROOT"

# Clean and create AppDir
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APPDIR/usr/share/icons/hicolor/128x128/apps"
mkdir -p "$APPDIR/usr/share/icons/hicolor/64x64/apps"
mkdir -p "$APPDIR/usr/share/icons/hicolor/48x48/apps"

# Build the release binary
echo "Building release binary..."
cd "$PROJECT_ROOT"
cargo build --release -p orpheus-app --features audio

# Copy binary
echo "Copying binary..."
cp "$PROJECT_ROOT/target/release/orpheus" "$APPDIR/usr/bin/"
strip "$APPDIR/usr/bin/orpheus"

# Copy desktop file
echo "Copying desktop file..."
cp "$SCRIPT_DIR/orpheus.desktop" "$APPDIR/usr/share/applications/"
cp "$SCRIPT_DIR/orpheus.desktop" "$APPDIR/"

# Copy/generate icon
echo "Setting up icons..."
if [ -f "$SCRIPT_DIR/orpheus-256.png" ]; then
    cp "$SCRIPT_DIR/orpheus-256.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/orpheus.png"
    cp "$SCRIPT_DIR/orpheus-256.png" "$APPDIR/orpheus.png"
else
    # Generate placeholder icon (a purple square for now)
    echo "No icon found, using placeholder..."
    convert -size 256x256 xc:'#7B2CBF' -fill white -gravity center \
        -font DejaVu-Sans-Bold -pointsize 120 -annotate 0 'O' \
        "$APPDIR/usr/share/icons/hicolor/256x256/apps/orpheus.png" 2>/dev/null || \
    echo "Install imagemagick to generate placeholder icon"
fi

# Create AppRun
echo "Creating AppRun..."
cat > "$APPDIR/AppRun" << 'APPRUN'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin/:${HERE}/usr/sbin/:${HERE}/usr/games/:${HERE}/bin/:${HERE}/sbin/:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib/:${HERE}/usr/lib/x86_64-linux-gnu/:${HERE}/usr/lib64/:${HERE}/lib/:${HERE}/lib/x86_64-linux-gnu/:${HERE}/lib64/:${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share/:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
exec "${HERE}/usr/bin/orpheus" "$@"
APPRUN
chmod +x "$APPDIR/AppRun"

# Create symlinks required by AppImage spec
ln -sf usr/share/applications/orpheus.desktop "$APPDIR/orpheus.desktop" 2>/dev/null || true

# Check for appimagetool
if ! command -v appimagetool &> /dev/null; then
    echo ""
    echo "=== AppDir created successfully ==="
    echo "Location: $APPDIR"
    echo ""
    echo "To create AppImage, install appimagetool and run:"
    echo "  appimagetool $APPDIR Orpheus-x86_64.AppImage"
    echo ""
    echo "Download appimagetool from:"
    echo "  https://github.com/AppImage/AppImageKit/releases"
    exit 0
fi

# Build AppImage
echo "Building AppImage..."
cd "$BUILD_DIR"
ARCH=x86_64 appimagetool "$APPDIR" "Orpheus-x86_64.AppImage"

echo ""
echo "=== AppImage built successfully ==="
echo "Output: $BUILD_DIR/Orpheus-x86_64.AppImage"
echo ""
echo "Test with:"
echo "  ./Orpheus-x86_64.AppImage"
