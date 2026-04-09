#!/usr/bin/env bash
# Trigger Reminders Automation prompt via a read-only AppleScript call.

set -euo pipefail

echo "Requesting Automation permission for Reminders..."

osascript -e 'tell application "Reminders"
  activate
  set listNames to name of every list
  return "Lists: " & (listNames as text)
end tell'

echo "Reminders responded. If prompted, grant Terminal/Python permission."
