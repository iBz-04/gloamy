#!/usr/bin/env python3
"""Trigger Notes Automation prompt via a read-only AppleScript call."""

import subprocess
import sys
from textwrap import dedent

APPLESCRIPT = dedent(
    """
    tell application "Notes"
        activate
        set accountNames to name of every account
        set folderNames to name of every folder
        return "Accounts: " & (accountNames as text) & " | Folders: " & (folderNames as text)
    end tell
    """
)


def main() -> int:
    print("Requesting Automation permission for Notes...")
    result = subprocess.run(
        ["osascript", "-e", APPLESCRIPT],
        capture_output=True,
        text=True,
    )

    if result.stdout.strip():
        print(result.stdout.strip())

    if result.returncode != 0:
        print(result.stderr.strip() or "Notes check failed without error output.")

    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
