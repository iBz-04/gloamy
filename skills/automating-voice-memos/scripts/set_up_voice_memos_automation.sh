#!/usr/bin/env bash
# Voice Memos setup helper: tries to activate the app (best effort) and reports the data path.
# Notes:
# - Voice Memos has no AppleScript dictionary. Automation relies on UI scripting and filesystem access.
# - You likely need Accessibility (for UI scripting) and Full Disk Access (for database/files).

set -euo pipefail

echo "Attempting to launch Voice Memos (best effort)..."

# Try bundle ID first, then a few common paths (Catalyst name is often VoiceMemos.app without a space).
launch_attempted=0
if open -b com.apple.VoiceMemos 2>/dev/null; then
  launch_attempted=1
else
  possible_paths=(
    "/System/Applications/VoiceMemos.app"
    "/System/Applications/Voice Memos.app"
    "/Applications/VoiceMemos.app"
    "/Applications/Voice Memos.app"
  )
  for app_path in "${possible_paths[@]}"; do
    if [[ -d "$app_path" ]]; then
      echo "Found app at: $app_path"
      if open "$app_path" 2>/dev/null || open -a "$app_path" 2>/dev/null; then
        launch_attempted=1
        break
      else
        echo "Launch attempt failed for $app_path"
      fi
    fi
  done
fi

if [[ "$launch_attempted" -eq 0 ]]; then
  echo "Warning: Could not launch Voice Memos via bundle ID or known paths; verify the app location."
fi

# Detect likely data paths across macOS versions.
paths=(
  "$HOME/Library/Group Containers/group.com.apple.VoiceMemos.shared/Recordings"
  "$HOME/Library/Application Support/com.apple.voicememos/Recordings"
  "$HOME/Library/Containers/com.apple.VoiceMemos/Data/Library/Application Support/Recordings"
)

echo "Checking known Voice Memos data locations:"
found=0
for p in "${paths[@]}"; do
  if [[ -d "$p" ]]; then
    echo "✔ Found: $p"
    found=1
  else
    echo "✖ Not present: $p"
  fi
done

if [[ "$found" -eq 0 ]]; then
  echo "No known data path found. Run Voice Memos once and re-run this script."
else
  echo "If you need direct file/database access, grant Full Disk Access to Terminal/Python."
fi

echo "Reminder: Enable Accessibility for UI scripting (System Settings > Privacy & Security > Accessibility)."
