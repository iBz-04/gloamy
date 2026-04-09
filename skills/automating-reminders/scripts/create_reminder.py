#!/usr/bin/env python3
"""
Create Reminder Script - PyXA Implementation
Creates a new reminder in Reminders app

Usage: python create_reminder.py "Reminder Title" ["due date"] ["list name"]
"""

import sys
from datetime import datetime, timedelta
import PyXA

def create_reminder(title, due_date_str=None, list_name="Reminders"):
    """Create a new reminder"""
    try:
        reminders = PyXA.Application("Reminders")

        # Parse due date if provided
        due_date = None
        if due_date_str:
            try:
                due_date = datetime.fromisoformat(due_date_str)
            except:
                print(f"Invalid date format: {due_date_str}. Use YYYY-MM-DD or YYYY-MM-DD HH:MM")
                return False

        # Find or create list
        target_list = None
        lists = reminders.lists()

        for lst in lists:
            if lst.name == list_name:
                target_list = lst
                break

        if not target_list:
            # Create new list
            target_list = reminders.new_list(list_name)

        # Create reminder
        reminder_data = {"name": title}
        if due_date:
            reminder_data["due_date"] = due_date

        reminder = target_list.reminders().push(reminder_data)

        print(f"Created reminder: '{title}'")
        if due_date:
            print(f"Due: {due_date.strftime('%Y-%m-%d %H:%M')}")
        print(f"List: {list_name}")

        return True

    except Exception as e:
        print(f"Error creating reminder: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python create_reminder.py 'Reminder Title' ['YYYY-MM-DD'] ['List Name']")
        sys.exit(1)

    title = sys.argv[1]
    due_date = sys.argv[2] if len(sys.argv) > 2 else None
    list_name = sys.argv[3] if len(sys.argv) > 3 else "Reminders"

    success = create_reminder(title, due_date, list_name)
    sys.exit(0 if success else 1)