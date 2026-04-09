#!/usr/bin/env python3
"""
List Upcoming Events Script - PyXA Implementation
Lists upcoming calendar events

Usage: python list_upcoming_events.py [days_ahead] [calendar_name]
"""

import sys
import PyXA
from datetime import datetime, timedelta

def list_upcoming_events(days_ahead=7, calendar_name=None):
    """List upcoming calendar events"""
    try:
        calendar_app = PyXA.Application("Calendar")

        # Calculate date range
        start_date = datetime.now()
        end_date = start_date + timedelta(days=days_ahead)

        # Get calendars to search
        calendars = []
        if calendar_name:
            # Find specific calendar
            for cal in calendar_app.calendars():
                if calendar_name.lower() in cal.name.lower():
                    calendars = [cal]
                    break
        else:
            # Use all calendars
            calendars = calendar_app.calendars()

        if not calendars:
            print(f"Calendar '{calendar_name}' not found" if calendar_name else "No calendars found")
            return []

        upcoming_events = []

        # Search each calendar
        for calendar in calendars:
            events = calendar.events()

            # Filter events in date range
            for event in events:
                event_start = event.start_date()
                if start_date <= event_start <= end_date:
                    upcoming_events.append({
                        'title': event.summary(),
                        'start': event_start,
                        'end': event.end_date(),
                        'calendar': calendar.name(),
                        'location': event.location() or ""
                    })

        # Sort by start time
        upcoming_events.sort(key=lambda x: x['start'])

        # Display results
        if upcoming_events:
            print(f"Upcoming events in the next {days_ahead} days:")
            print("=" * 50)

            for event in upcoming_events:
                start_str = event['start'].strftime('%Y-%m-%d %H:%M')
                end_str = event['end'].strftime('%H:%M')
                print(f"📅 {event['title']}")
                print(f"   🕐 {start_str} - {end_str}")
                print(f"   📍 {event['calendar']}")
                if event['location']:
                    print(f"   📌 {event['location']}")
                print()
        else:
            print(f"No upcoming events found in the next {days_ahead} days.")

        return upcoming_events

    except Exception as e:
        print(f"Error listing upcoming events: {e}")
        return []

if __name__ == "__main__":
    days = int(sys.argv[1]) if len(sys.argv) > 1 else 7
    calendar = sys.argv[2] if len(sys.argv) > 2 else None

    events = list_upcoming_events(days, calendar)
    sys.exit(0 if events else 1)