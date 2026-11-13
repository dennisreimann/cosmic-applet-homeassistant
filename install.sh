#!/usr/bin/env bash
# Installs from either a source checkout or an extracted release archive.
set -euo pipefail
APP_ID="net.d11n.CosmicHomeassistant"
BIN_NAME="cosmic-applet-homeassistant"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINDIR="${BINDIR:-$HOME/.local/bin}"
DATADIR="${DATADIR:-$HOME/.local/share}"
BIN="$ROOT/$BIN_NAME"
[ -x "$BIN" ] || BIN="$ROOT/target/release/$BIN_NAME"
SETTINGS_BIN="$ROOT/$BIN_NAME-settings"
[ -x "$SETTINGS_BIN" ] || SETTINGS_BIN="$ROOT/target/release/$BIN_NAME-settings"
DESKTOP="$ROOT/data/$APP_ID.desktop"
[ -f "$DESKTOP" ] || DESKTOP="$ROOT/$APP_ID.desktop"
ICON="$ROOT/assets/ha.svg"
for file in "$BIN" "$SETTINGS_BIN" "$DESKTOP" "$ICON"; do
    [ -f "$file" ] || { echo "Error: $file not found; build with cargo build --release" >&2; exit 1; }
done
install -Dm0755 "$BIN" "$BINDIR/$BIN_NAME"
install -Dm0755 "$SETTINGS_BIN" "$BINDIR/$BIN_NAME-settings"
install -Dm0644 "$ICON" "$DATADIR/icons/hicolor/scalable/apps/$APP_ID.svg"
mkdir -p "$DATADIR/applications"
sed "s|@BIN@|$BINDIR/$BIN_NAME|" "$DESKTOP" > "$DATADIR/applications/$APP_ID.desktop"
gtk-update-icon-cache -f -t "$DATADIR/icons/hicolor" 2>/dev/null || true
echo "Installed Home Assistant in $BINDIR. Add it in Settings -> Panel -> Applets."
