#!/usr/bin/env python3
"""
List Reminders Script - PyXA Implementation
Lists reminders from Reminders app with filtering options

Usage: python list_reminders.py [--list "List Name"] [--completed] [--overdue]
"""

import sys
import PyXA
from datetime import datetime

def list_reminders(list_name=None, show_completed=False, show_overdue_only=False):
    """List reminders with optional filtering"""
    try:
        reminders_app = PyXA.Application("Reminders")

        all_lists = reminders_app.lists()
        total_reminders = 0

        for lst in all_lists:
            # Skip if specific list requested and this isn't it
            if list_name and lst.name != list_name:
                continue

            list_reminders = lst.reminders()
            filtered_reminders = []

            for reminder in list_reminders:
                # Filter by completion status
                if not show_completed and reminder.completed():
                    continue

                # Filter by overdue status
                if show_overdue_only:
                    due_date = reminder.due_date()
                    if not due_date or due_date > datetime.now():
                        continue

                filtered_reminders.append(reminder)

            if filtered_reminders:
                print(f"\n📝 {lst.name} ({len(filtered_reminders)} reminders):")
                print("-" * 50)

                for reminder in sorted(filtered_reminders,
                                     key=lambda r: r.due_date() or datetime.max):
                    title = reminder.name()
                    completed = "✅" if reminder.completed() else "⏳"
                    due_date = reminder.due_date()

                    print(f"  {completed} {title}")

                    if due_date:
                        if due_date < datetime.now() and not reminder.completed():
                            print(f"      🚨 Overdue: {due_date.strftime('%Y-%m-%d %H:%M')}")
                        else:
                            print(f"      📅 Due: {due_date.strftime('%Y-%m-%d %H:%M')}")

                    total_reminders += 1

        if total_reminders == 0:
            filters = []
            if list_name:
                filters.append(f"list '{list_name}'")
            if not show_completed:
                filters.append("incomplete only")
            if show_overdue_only:
                filters.append("overdue only")

            filter_desc = f" ({', '.join(filters)})" if filters else ""
            print(f"No reminders found{filter_desc}")
        else:
            print(f"\n📊 Total: {total_reminders} reminders")

        return total_reminders

    except Exception as e:
        print(f"Error listing reminders: {e}")
        return 0

if __name__ == "__main__":
    list_name = None
    show_completed = False
    show_overdue = False

    # Parse arguments
    for arg in sys.argv[1:]:
        if arg.startswith('--list='):
            list_name = arg.split('=', 1)[1]
        elif arg == '--list' and len(sys.argv) > sys.argv.index(arg) + 1:
            list_name = sys.argv[sys.argv.index(arg) + 1]
        elif arg == '--completed':
            show_completed = True
        elif arg == '--overdue':
            show_overdue = True

    count = list_reminders(list_name, show_completed, show_overdue)
    sys.exit(0 if count >= 0 else 1)