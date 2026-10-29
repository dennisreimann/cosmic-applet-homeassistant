#!/usr/bin/env bash
# Installs the applet user-locally (binary + applet discovery).
set -euo pipefail

APP_ID="net.d11n.CosmicHomeassistant"
BIN_NAME="cosmic-applet-homeassistant"
ROOT="$(cd "$(dirname "$0")" && pwd)"
BIN="$ROOT/target/release/$BIN_NAME"

[ -x "$BIN" ] || { echo "Error: $BIN not found — run 'cargo build --release' first" >&2; exit 1; }

install -Dm0755 "$BIN" "$HOME/.local/bin/$BIN_NAME"
SETTINGS_BIN="$ROOT/target/release/cosmic-applet-homeassistant-settings"
[ -x "$SETTINGS_BIN" ] || { echo "Error: $SETTINGS_BIN not found — run 'cargo build --release' first" >&2; exit 1; }
install -Dm0755 "$SETTINGS_BIN" "$HOME/.local/bin/cosmic-applet-homeassistant-settings"
mkdir -p "$HOME/.local/share/applications"
sed "s|@BIN@|$HOME/.local/bin/$BIN_NAME|" "data/$APP_ID.desktop" \
    > "$HOME/.local/share/applications/$APP_ID.desktop"

# Icon für die Applet-Liste in den Systemeinstellungen (ohne Icon wird
# der Eintrag dort nicht angezeigt).
mkdir -p "$HOME/.local/share/icons/hicolor/scalable/apps"
cp "$ROOT/assets/ha.svg" \
    "$HOME/.local/share/icons/hicolor/scalable/apps/$APP_ID.svg"

gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true

echo "Installed. Add the 'Home Assistant' applet in COSMIC Panel settings:"
echo "  Settings -> Panel -> Applets (right wing)"
