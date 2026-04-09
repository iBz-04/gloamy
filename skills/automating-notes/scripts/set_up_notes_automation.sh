#!/usr/bin/env bash
# Trigger Notes Automation prompt via a read-only AppleScript call.

set -euo pipefail

echo "Requesting Automation permission for Notes..."

osascript -e 'tell application "Notes"
  activate
  set accountNames to name of every account
  set folderNames to name of every folder
  return "Accounts: " & (accountNames as text) & " | Folders: " & (folderNames as text)
end tell'

echo "Notes responded. If prompted, grant Terminal/Python permission."
