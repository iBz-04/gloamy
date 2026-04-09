#!/usr/bin/env bash
# Trigger Mail Automation prompt via a read-only AppleScript call.

set -euo pipefail

echo "Requesting Automation permission for Mail..."

osascript -e 'tell application "Mail"
  activate
  set accountNames to name of every account
  set inboxNames to name of every mailbox of inbox
  return "Accounts: " & (accountNames as text) & " | Inbox mailboxes: " & (inboxNames as text)
end tell'

echo "Mail responded. If prompted, grant Terminal/Python permission."
