#!/usr/bin/env python3
"""
Run a read-only AppleScript against common Mac apps to trigger Automation prompts.
This helps grant Terminal/Python permission to control each app before running real automations.
"""

import subprocess
import sys
from textwrap import dedent


APP_SCRIPTS = {
    "Calendar": dedent(
        """
        tell application "Calendar"
            activate
            set calendarNames to name of every calendar
            return "Calendars: " & (calendarNames as text)
        end tell
        """
    ),
    "Notes": dedent(
        """
        tell application "Notes"
            activate
            set accountNames to name of every account
            set folderNames to name of every folder
            return "Accounts: " & (accountNames as text) & " | Folders: " & (folderNames as text)
        end tell
        """
    ),
    "Mail": dedent(
        """
        tell application "Mail"
            activate
            set accountNames to name of every account
            set inboxNames to name of every mailbox of inbox
            return "Accounts: " & (accountNames as text) & " | Inbox mailboxes: " & (inboxNames as text)
        end tell
        """
    ),
    "Keynote": dedent(
        """
        tell application "Keynote"
            activate
            set docNames to name of documents
            return "Open documents: " & (docNames as text)
        end tell
        """
    ),
    "Microsoft Excel": dedent(
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
    ),
    "Reminders": dedent(
        """
        tell application "Reminders"
            activate
            set listNames to name of every list
            return "Lists: " & (listNames as text)
        end tell
        """
    ),
}


def run_applescript(app_name: str, script: str) -> tuple[int, str, str]:
    """Run an AppleScript snippet for a specific app via osascript."""
    result = subprocess.run(
        ["osascript", "-e", script],
        capture_output=True,
        text=True,
    )
    stdout = result.stdout.strip()
    stderr = result.stderr.strip()
    if stdout:
        print(f"{app_name} responded: {stdout}")
    if result.returncode != 0:
        print(f"{app_name} failed ({result.returncode}): {stderr or 'no error output'}")
    return result.returncode, stdout, stderr


def main() -> int:
    print("Triggering Automation prompts for common apps (read-only commands).")
    failures = []

    for app_name, script in APP_SCRIPTS.items():
        print(f"\n=== {app_name} ===")
        code, _, _ = run_applescript(app_name, script)
        if code != 0:
            failures.append(app_name)

    if failures:
        print("\nSome apps did not respond cleanly. Re-run those after granting permissions:")
        for name in failures:
            print(f"- {name}")
        return 1

    print("\nAutomation prompts completed for all apps.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
