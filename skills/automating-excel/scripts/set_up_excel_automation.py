#!/usr/bin/env python3
"""Trigger Excel Automation prompt via a read-only AppleScript call."""

import subprocess
import sys
from textwrap import dedent

APPLESCRIPT = dedent(
    """
    tell application "Microsoft Excel"
        activate
        set workbookCount to count of workbooks
        if workbookCount is 0 then
            return "No workbooks open; launch Excel and open/create one to confirm."
        else
            set workbookNames to name of workbooks
            return "Open workbooks: " & (workbookNames as text)
        end if
    end tell
    """
)


def main() -> int:
    print("Requesting Automation permission for Microsoft Excel...")
    result = subprocess.run(
        ["osascript", "-e", APPLESCRIPT],
        capture_output=True,
        text=True,
    )

    if result.stdout.strip():
        print(result.stdout.strip())

    if result.returncode != 0:
        print(result.stderr.strip() or "Excel check failed without error output.")

    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
