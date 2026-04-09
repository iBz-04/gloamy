#!/usr/bin/env bash
# Trigger Excel Automation prompt via a read-only AppleScript call.

set -euo pipefail

echo "Requesting Automation permission for Microsoft Excel..."

osascript -e 'tell application "Microsoft Excel"
  activate
  set workbookCount to count of workbooks
  if workbookCount is 0 then
    return "No workbooks open; launch Excel and open/create one to confirm."
  else
    set workbookNames to name of workbooks
    return "Open workbooks: " & (workbookNames as text)
  end if
end tell'

echo "Excel responded. If prompted, grant Terminal/Python permission."
