#!/bin/bash
set -e

APP_NAME="waylyrics"
APP_ID="io.github.waylyrics.Waylyrics"
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
ARCH="x86_64"

echo "Building AppImage for ${APP_NAME} v${VERSION}"

# Build release binary (without THEME_PRESETS_DIR, will use user theme dir)
echo "Building release binary..."
cargo build --release

# Create AppDir structure
APPDIR="AppDir"
rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${APPDIR}/usr/share/metainfo"

# Copy binary
cp target/release/waylyrics "${APPDIR}/usr/bin/"

# Create desktop file
cat > "${APPDIR}/usr/share/applications/${APP_ID}.desktop" << 'EOF'
[Desktop Entry]
Type=Application
Version=1.0
Name=Waylyrics
Comment=desktop lyrics with QQMusic/NetEase Cloud Music source
Exec=waylyrics
Icon=io.github.waylyrics.Waylyrics
Terminal=false
Categories=Audio;AudioVideo;GTK;Player;
EOF

# Copy desktop file and icon to AppDir root (required by appimagetool)
cp "${APPDIR}/usr/share/applications/${APP_ID}.desktop" "${APPDIR}/"

# Copy icon (use SVG if available, otherwise use PNG)
if [ -f "res/icons/hicolor/scalable/apps/io.github.waylyrics.Waylyrics.svg" ]; then
    cp "res/icons/hicolor/scalable/apps/io.github.waylyrics.Waylyrics.svg" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/"
    cp "res/icons/hicolor/scalable/apps/io.github.waylyrics.Waylyrics.svg" "${APPDIR}/"
elif [ -f "res/icons/hicolor/256x256/apps/io.github.waylyrics.Waylyrics.png" ]; then
    mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"
    cp "res/icons/hicolor/256x256/apps/io.github.waylyrics.Waylyrics.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/"
    cp "res/icons/hicolor/256x256/apps/io.github.waylyrics.Waylyrics.png" "${APPDIR}/"
fi

# Copy and compile GSettings schema
mkdir -p "${APPDIR}/usr/share/glib-2.0/schemas"
cp metainfo/io.github.waylyrics.Waylyrics.gschema.xml "${APPDIR}/usr/share/glib-2.0/schemas/"
if command -v glib-compile-schemas &> /dev/null; then
    glib-compile-schemas "${APPDIR}/usr/share/glib-2.0/schemas/"
    echo "GSettings schema compiled"
else
    echo "WARNING: glib-compile-schemas not found"
fi

# Copy metainfo
cp metainfo/io.github.waylyrics.Waylyrics.metainfo.xml "${APPDIR}/usr/share/metainfo/"

# Copy themes
mkdir -p "${APPDIR}/usr/share/waylyrics/themes"
cp themes/*.css "${APPDIR}/usr/share/waylyrics/themes/"

# Compile and install locale files
if command -v msgfmt &> /dev/null; then
    echo "Compiling locale files..."
    for po in locales/*/LC_MESSAGES/waylyrics.po; do
        lang=$(echo "$po" | sed 's|locales/\([^/]*\)/.*|\1|')
        mkdir -p "${APPDIR}/usr/share/locale/${lang}/LC_MESSAGES"
        msgfmt -o "${APPDIR}/usr/share/locale/${lang}/LC_MESSAGES/waylyrics.mo" "$po"
        echo "  Compiled: ${lang}"
    done
else
    echo "WARNING: msgfmt not found, locale files not compiled"
fi

# Create AppRun script
cat > "${APPDIR}/AppRun" << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
export TEXTDOMAINDIR="${HERE}/usr/share/locale"

# Copy themes to user theme directory if not exists
USER_THEME_DIR="${HOME}/.local/share/waylyrics/_themes"
if [ ! -d "${USER_THEME_DIR}" ] || [ -z "$(ls -A "${USER_THEME_DIR}" 2>/dev/null)" ]; then
    mkdir -p "${USER_THEME_DIR}"
    cp "${HERE}/usr/share/waylyrics/themes/"*.css "${USER_THEME_DIR}/" 2>/dev/null || true
fi

exec waylyrics "$@"
EOF
chmod +x "${APPDIR}/AppRun"

echo "AppDir created at ${APPDIR}"

# Create AppImage if appimagetool is available
if command -v appimagetool &> /dev/null; then
    echo "Creating AppImage..."
    ARCH=${ARCH} appimagetool "${APPDIR}" "${APP_NAME}-${VERSION}-${ARCH}.AppImage"
    echo "AppImage created: ${APP_NAME}-${VERSION}-${ARCH}.AppImage"
else
    echo "appimagetool not found. To create AppImage, run:"
    echo "  ARCH=${ARCH} appimagetool ${APPDIR} ${APP_NAME}-${VERSION}-${ARCH}.AppImage"
fi
