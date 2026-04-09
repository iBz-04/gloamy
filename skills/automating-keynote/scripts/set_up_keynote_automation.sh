#!/usr/bin/env bash
# Trigger Keynote Automation prompt via a read-only AppleScript call.

set -euo pipefail

echo "Requesting Automation permission for Keynote..."

osascript -e 'tell application "Keynote"
  activate
  set docNames to name of documents
  return "Open documents: " & (docNames as text)
end tell'

echo "Keynote responded. If prompted, grant Terminal/Python permission."
