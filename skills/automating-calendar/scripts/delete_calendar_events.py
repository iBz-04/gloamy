#!/usr/bin/env python3
"""
Delete Calendar Events Script - PyXA Implementation
Deletes calendar events based on criteria

Usage: python delete_calendar_events.py "event title pattern" [--dry-run]
"""

import sys
import PyXA

def delete_calendar_events(title_pattern, dry_run=False):
    """Delete calendar events matching the title pattern"""
    try:
        calendar_app = PyXA.Application("Calendar")

        # Get all calendars
        calendars = calendar_app.calendars()

        deleted_count = 0
        found_events = []

        # Search through all calendars
        for calendar in calendars:
            events = calendar.events()

            # Find events matching the pattern
            matching_events = []
            for event in events:
                title = event.summary() or ""
                if title_pattern.lower() in title.lower():
                    matching_events.append(event)

            for event in matching_events:
                found_events.append({
                    'title': event.summary(),
                    'start': event.start_date(),
                    'calendar': calendar.name()
                })

                if not dry_run:
                    try:
                        event.delete()
                        deleted_count += 1
                        print(f"Deleted: {event.summary()}")
                    except Exception as e:
                        print(f"Failed to delete '{event.summary()}': {e}")
                else:
                    print(f"Would delete: {event.summary()}")

        if dry_run:
            print(f"\nDry run complete. Found {len(found_events)} matching events.")
        else:
            print(f"\nDeleted {deleted_count} events matching '{title_pattern}'")

        return found_events

    except Exception as e:
        print(f"Error deleting calendar events: {e}")
        return []

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python delete_calendar_events.py 'title pattern' [--dry-run]")
        sys.exit(1)

    title_pattern = sys.argv[1]
    dry_run = "--dry-run" in sys.argv

    events = delete_calendar_events(title_pattern, dry_run)
    sys.exit(0 if events else 1)