#!/usr/bin/env bash
# Trigger Calendar Automation prompt via a read-only AppleScript call.

set -euo pipefail

echo "Requesting Automation permission for Calendar..."

osascript -e 'tell application "Calendar"
  activate
  set accountNames to name of every calendar account
  set calendarNames to name of every calendar
  return "Accounts: " & (accountNames as text) & " | Calendars: " & (calendarNames as text)
end tell'

echo "Calendar responded. If prompted, grant Terminal/Python permission."
