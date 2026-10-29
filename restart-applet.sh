#!/usr/bin/env bash
# Restarts the cosmic-applet-homeassistant applet.
#
# The panel does not automatically restart applets that were killed (only on
# crashes during a restart sequence). This script therefore toggles the applet
# entry in the panel configuration, which the panel processes as a hot reload.
set -euo pipefail

APP_ID="net.d11n.CosmicHomeassistant"
BIN_NAME="cosmic-applet-homeassistant"
CONF="$HOME/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings"

[ -f "$CONF" ] || { echo "Error: panel config not found: $CONF" >&2; exit 1; }

# Finds PIDs via the real binary path (/proc/<pid>/exe) — robust against
# cmdline matches (e.g. this very script invocation).
find_applet_pids() {
    for pid in $(pgrep -f "$BIN_NAME" || true); do
        local exe
        exe=$(readlink "/proc/$pid/exe" 2>/dev/null || true)
        case "$exe" in
            */"$BIN_NAME") echo "$pid" ;;
        esac
    done
}

# 1) Stop the running applet process.
for pid in $(find_applet_pids); do
    kill "$pid" 2>/dev/null || true
done
sleep 1

# 2) Remove the applet entry from the config.
python3 - "$CONF" "$APP_ID" <<'PYEOF'
import sys
conf, app_id = sys.argv[1], sys.argv[2]
lines = open(conf).read().splitlines(keepends=True)
lines = [l for l in lines if app_id not in l]
open(conf, "w").writelines(lines)
PYEOF
sleep 2

# 3) Re-insert the applet entry (right wing, in front of the StatusArea).
python3 - "$CONF" "$APP_ID" <<'PYEOF'
import sys
conf, app_id = sys.argv[1], sys.argv[2]
lines = open(conf).read().splitlines(keepends=True)
for i, l in enumerate(lines):
    if '"com.system76.CosmicAppletStatusArea",' in l:
        lines.insert(i, f'    "{app_id}",\n')
        break
else:
    # Fallback: append as the last entry of the right wing list.
    for i, l in enumerate(lines):
        if l.strip() == "]))":
            lines.insert(i, f'    "{app_id}",\n')
            break
open(conf, "w").writelines(lines)
PYEOF
sleep 4

# 4) Verify.
if [ -n "$(find_applet_pids)" ]; then
    echo "Applet is running again."
else
    echo "Applet was not started — check the panel config: $CONF" >&2
    exit 1
fi
