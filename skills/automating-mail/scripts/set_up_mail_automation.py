#!/usr/bin/env python3
"""Trigger Mail Automation prompt via a read-only AppleScript call."""

import subprocess
import sys
from textwrap import dedent

APPLESCRIPT = dedent(
    """
    tell application "Mail"
        activate
        set accountNames to name of every account
        set inboxNames to name of every mailbox of inbox
        return "Accounts: " & (accountNames as text) & " | Inbox mailboxes: " & (inboxNames as text)
    end tell
    """
)


def main() -> int:
    print("Requesting Automation permission for Mail...")
    result = subprocess.run(
        ["osascript", "-e", APPLESCRIPT],
        capture_output=True,
        text=True,
    )

    if result.stdout.strip():
        print(result.stdout.strip())

    if result.returncode != 0:
        print(result.stderr.strip() or "Mail check failed without error output.")

    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
