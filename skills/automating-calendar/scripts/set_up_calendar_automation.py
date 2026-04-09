#!/usr/bin/env python3
"""Trigger Calendar Automation prompt via a read-only AppleScript call."""

import subprocess
import sys
from textwrap import dedent

APPLESCRIPT = dedent(
    """
    tell application "Calendar"
        activate
        set accountNames to name of every calendar account
        set calendarNames to name of every calendar
        return "Accounts: " & (accountNames as text) & " | Calendars: " & (calendarNames as text)
    end tell
    """
)


def main() -> int:
    print("Requesting Automation permission for Calendar...")
    result = subprocess.run(
        ["osascript", "-e", APPLESCRIPT],
        capture_output=True,
        text=True,
    )

    if result.stdout.strip():
        print(result.stdout.strip())

    if result.returncode != 0:
        print(result.stderr.strip() or "Calendar check failed without error output.")

    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
