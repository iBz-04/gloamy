#!/usr/bin/env python3
"""
Find Free Time Slots Script - PyXA Implementation
Finds available time slots in calendar for scheduling

Usage: python find_free_slots.py "2024-01-15" [--duration 60] [--calendar "Work"]
"""

import sys
import PyXA
from datetime import datetime, timedelta, time

def find_free_slots(date_str, duration_minutes=60, calendar_name=None):
    """Find free time slots on a given date"""
    try:
        calendar_app = PyXA.Application("Calendar")

        # Parse date
        target_date = datetime.fromisoformat(date_str).date()

        # Get calendars to check
        calendars = []
        if calendar_name:
            for cal in calendar_app.calendars():
                if calendar_name.lower() in cal.name.lower():
                    calendars = [cal]
                    break
        else:
            calendars = calendar_app.calendars()

        if not calendars:
            print(f"Calendar '{calendar_name}' not found" if calendar_name else "No calendars found")
            return []

        # Define workday (9 AM to 5 PM)
        workday_start = time(9, 0)
        workday_end = time(17, 0)

        # Create datetime objects for the day
        day_start = datetime.combine(target_date, workday_start)
        day_end = datetime.combine(target_date, workday_end)

        # Collect all events for the day
        day_events = []

        for calendar in calendars:
            events = calendar.events()
            for event in events:
                event_start = event.start_date()
                event_end = event.end_date()

                # Check if event overlaps with target day
                if (event_start.date() == target_date or
                    event_end.date() == target_date or
                    (event_start.date() < target_date and event_end.date() > target_date)):

                    # Adjust for multi-day events
                    actual_start = max(event_start, day_start)
                    actual_end = min(event_end, day_end)

                    if actual_start < actual_end:
                        day_events.append({
                            'start': actual_start,
                            'end': actual_end,
                            'title': event.summary()
                        })

        # Sort events by start time
        day_events.sort(key=lambda x: x['start'])

        # Find free slots
        free_slots = []
        current_time = day_start

        for event in day_events:
            if current_time < event['start']:
                # There's free time before this event
                slot_duration = (event['start'] - current_time).total_seconds() / 60
                if slot_duration >= duration_minutes:
                    free_slots.append({
                        'start': current_time,
                        'end': event['start'],
                        'duration_minutes': slot_duration
                    })

            # Move current time to end of event
            current_time = max(current_time, event['end'])

        # Check for free time after last event
        if current_time < day_end:
            slot_duration = (day_end - current_time).total_seconds() / 60
            if slot_duration >= duration_minutes:
                free_slots.append({
                    'start': current_time,
                    'end': day_end,
                    'duration_minutes': slot_duration
                })

        # Display results
        if free_slots:
            print(f"Free {duration_minutes}-minute slots on {target_date.strftime('%Y-%m-%d')}:")
            for i, slot in enumerate(free_slots, 1):
                start_str = slot['start'].strftime('%H:%M')
                end_str = slot['end'].strftime('%H:%M')
                print(f"{i}. {start_str} - {end_str} ({slot['duration_minutes']:.0f} minutes)")
        else:
            print(f"No free {duration_minutes}-minute slots found on {target_date.strftime('%Y-%m-%d')}")

        return free_slots

    except Exception as e:
        print(f"Error finding free slots: {e}")
        return []

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python find_free_slots.py '2024-01-15' [--duration 60] [--calendar 'Work']")
        sys.exit(1)

    date_str = sys.argv[1]
    duration = 60
    calendar = None

    # Parse optional arguments
    for arg in sys.argv[2:]:
        if arg.startswith('--duration'):
            duration = int(arg.split('=')[1]) if '=' in arg else int(sys.argv[sys.argv.index(arg) + 1])
        elif arg.startswith('--calendar'):
            calendar = arg.split('=')[1] if '=' in arg else sys.argv[sys.argv.index(arg) + 1]

    slots = find_free_slots(date_str, duration, calendar)
    sys.exit(0 if slots else 1)