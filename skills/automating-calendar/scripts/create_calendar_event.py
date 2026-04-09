#!/usr/bin/env python3
"""
Create Calendar Event Script - PyXA Implementation
Creates a new calendar event

Usage: python create_calendar_event.py "Event Title" "2024-01-15 10:00" "2024-01-15 11:00" ["Event Location"]
"""

import sys
import PyXA
from datetime import datetime

def create_calendar_event(title, start_time_str, end_time_str, location=None):
    """Create a new calendar event"""
    try:
        calendar_app = PyXA.Application("Calendar")

        # Parse datetime strings
        start_time = datetime.fromisoformat(start_time_str)
        end_time = datetime.fromisoformat(end_time_str)

        # Get first calendar (usually default)
        default_calendar = calendar_app.calendars()[0]

        # Create event
        event = default_calendar.events().push({
            "summary": title,
            "start_date": start_time,
            "end_date": end_time,
            "location": location or ""
        })

        print(f"Created event: {title}")
        print(f"Time: {start_time.strftime('%Y-%m-%d %H:%M')} - {end_time.strftime('%H:%M')}")
        if location:
            print(f"Location: {location}")

        return True

    except Exception as e:
        print(f"Error creating calendar event: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: python create_calendar_event.py 'Event Title' '2024-01-15 10:00' '2024-01-15 11:00' ['Event Location']")
        sys.exit(1)

    title = sys.argv[1]
    start_time = sys.argv[2]
    end_time = sys.argv[3]
    location = sys.argv[4] if len(sys.argv) > 4 else None

    success = create_calendar_event(title, start_time, end_time, location)
    sys.exit(0 if success else 1)