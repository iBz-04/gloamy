#!/usr/bin/env python3
"""Trigger Keynote Automation prompt via a read-only AppleScript call."""

import subprocess
import sys
from textwrap import dedent

APPLESCRIPT = dedent(
    """
    tell application "Keynote"
        activate
        set docNames to name of documents
        return "Open documents: " & (docNames as text)
    end tell
    """
)


def main() -> int:
    print("Requesting Automation permission for Keynote...")
    result = subprocess.run(
        ["osascript", "-e", APPLESCRIPT],
        capture_output=True,
        text=True,
    )

    if result.stdout.strip():
        print(result.stdout.strip())

    if result.returncode != 0:
        print(result.stderr.strip() or "Keynote check failed without error output.")

    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
