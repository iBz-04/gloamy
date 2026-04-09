#!/usr/bin/env python3
"""
Complete Reminders Script - PyXA Implementation
Marks reminders as completed based on criteria

Usage: python complete_reminders.py "search pattern" [--list "List Name"] [--dry-run]
"""

import sys
import PyXA

def complete_reminders(search_pattern, list_name=None, dry_run=False):
    """Mark reminders as completed based on search pattern"""
    try:
        reminders_app = PyXA.Application("Reminders")

        all_lists = reminders_app.lists()
        completed_count = 0
        found_count = 0

        for lst in all_lists:
            # Skip if specific list requested and this isn't it
            if list_name and lst.name != list_name:
                continue

            list_reminders = lst.reminders()

            for reminder in list_reminders:
                title = reminder.name() or ""

                # Check if reminder matches search pattern and is not already completed
                if (search_pattern.lower() in title.lower() and
                    not reminder.completed()):

                    found_count += 1

                    if dry_run:
                        print(f"Would complete: '{title}' in {lst.name}")
                    else:
                        try:
                            reminder.completed = True
                            completed_count += 1
                            print(f"Completed: '{title}' in {lst.name}")
                        except Exception as e:
                            print(f"Error completing '{title}': {e}")

        if dry_run:
            print(f"\nDry run: Found {found_count} matching incomplete reminders")
        else:
            print(f"\nCompleted {completed_count} reminders matching '{search_pattern}'")

        return completed_count if not dry_run else found_count

    except Exception as e:
        print(f"Error completing reminders: {e}")
        return 0

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python complete_reminders.py 'search pattern' [--list 'List Name'] [--dry-run]")
        sys.exit(1)

    search_pattern = sys.argv[1]
    list_name = None
    dry_run = False

    # Parse optional arguments
    for arg in sys.argv[2:]:
        if arg.startswith('--list='):
            list_name = arg.split('=', 1)[1]
        elif arg == '--list' and len(sys.argv) > sys.argv.index(arg) + 1:
            list_name = sys.argv[sys.argv.index(arg) + 1]
        elif arg == '--dry-run':
            dry_run = True

    count = complete_reminders(search_pattern, list_name, dry_run)
    sys.exit(0 if count > 0 else 1)