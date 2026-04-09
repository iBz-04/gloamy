#!/usr/bin/env bash
# Trigger Automation prompts for common Mac apps using read-only AppleScript calls.
# Run this once after enabling Terminal/Python in System Settings > Privacy & Security > Automation.
# Run from the same host you intend to automate with (e.g., Terminal vs Python) so the right app is approved.

set -uo pipefail

run_script() {
  local app_name="$1"
  local script="$2"
  echo ""
  echo "=== ${app_name} ==="
  if ! osascript -e "$script"; then
    echo "Warning: ${app_name} script reported an error; check System Settings > Privacy & Security > Automation and retry."
    return 1
  fi
}

failures=0

run_script "Calendar" 'tell application "Calendar"
  activate
  set calendarNames to name of every calendar
  return "Calendars: " & (calendarNames as text)
end tell'
failures=$(( failures + $? ))

run_script "Notes" 'tell application "Notes"
  activate
  set accountNames to name of every account
  set folderNames to name of every folder
  return "Accounts: " & (accountNames as text) & " | Folders: " & (folderNames as text)
end tell'
failures=$(( failures + $? ))

run_script "Mail" 'tell application "Mail"
  activate
  set accountNames to name of every account
  set inboxNames to name of every mailbox of inbox
  return "Accounts: " & (accountNames as text) & " | Inbox mailboxes: " & (inboxNames as text)
end tell'
failures=$(( failures + $? ))

run_script "Keynote" 'tell application "Keynote"
  activate
  set docNames to name of documents
  return "Open documents: " & (docNames as text)
end tell'
failures=$(( failures + $? ))

run_script "Microsoft Excel" 'tell application "Microsoft Excel"
  activate
  set workbookCount to count of workbooks
  if workbookCount is 0 then
    return "No workbooks open; launch Excel and open/ create one to confirm."
  else
    set workbookNames to name of workbooks
    return "Open workbooks: " & (workbookNames as text)
  end if
end tell'
failures=$(( failures + $? ))

run_script "Reminders" 'tell application "Reminders"
  activate
  set listNames to name of every list
  return "Lists: " & (listNames as text)
end tell'
failures=$(( failures + $? ))

echo ""
if [[ "$failures" -eq 0 ]]; then
  echo "Automation prompts completed."
else
  echo "Automation prompts finished with ${failures} issue(s). Re-run the failing app(s) after granting permissions."
fi
