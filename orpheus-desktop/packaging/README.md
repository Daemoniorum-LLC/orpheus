# Orpheus Packaging

Platform-specific packaging scripts for distributing Orpheus.

## Linux (AppImage)

The recommended distribution format for Linux is AppImage, which provides a
single executable file that works across most Linux distributions.

### Build Requirements

- Rust toolchain (cargo)
- `libasound2-dev` for audio support
- `appimagetool` (optional, for creating final AppImage)

### Building

```bash
# Build AppDir (intermediate format)
./packaging/appimage/build-appimage.sh

# The script will create the AppImage if appimagetool is available
# Otherwise, it creates an AppDir that can be packaged manually
```

### Testing

```bash
# Run the AppImage directly
./target/appimage/Orpheus-x86_64.AppImage

# Or run from AppDir
./target/appimage/Orpheus.AppDir/AppRun
```

### Getting appimagetool

```bash
# Download
wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage

# Make executable
chmod +x appimagetool-x86_64.AppImage

# Move to PATH
sudo mv appimagetool-x86_64.AppImage /usr/local/bin/appimagetool
```

## macOS (App Bundle)

*Coming soon*

macOS distribution will use a `.app` bundle with code signing.

## Windows (Installer)

*Coming soon*

Windows distribution will use an MSI or NSIS installer.

## Flatpak

*Planned*

Flatpak support is planned for sandboxed Linux distribution.
