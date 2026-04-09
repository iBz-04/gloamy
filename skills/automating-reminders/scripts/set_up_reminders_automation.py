#!/usr/bin/env python3
"""Trigger Reminders Automation prompt via a read-only AppleScript call."""

import subprocess
import sys
from textwrap import dedent

APPLESCRIPT = dedent(
    """
    tell application "Reminders"
        activate
        set listNames to name of every list
        return "Lists: " & (listNames as text)
    end tell
    """
)


def main() -> int:
    print("Requesting Automation permission for Reminders...")
    result = subprocess.run(
        ["osascript", "-e", APPLESCRIPT],
        capture_output=True,
        text=True,
    )

    if result.stdout.strip():
        print(result.stdout.strip())

    if result.returncode != 0:
        print(result.stderr.strip() or "Reminders check failed without error output.")

    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
