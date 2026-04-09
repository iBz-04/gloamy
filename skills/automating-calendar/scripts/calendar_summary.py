#!/usr/bin/env python3
"""
Calendar Event Summary Script - PyXA Implementation
Generates a summary report of calendar events for a date range

Usage: python calendar_summary.py "2024-01-01" "2024-01-31" [--calendar "Work"]
"""

import sys
import PyXA
from datetime import datetime, timedelta
from collections import defaultdict

def generate_calendar_summary(start_date_str, end_date_str, calendar_name=None):
    """Generate a summary report of calendar events"""
    try:
        calendar_app = PyXA.Application("Calendar")

        # Parse dates
        start_date = datetime.fromisoformat(start_date_str)
        end_date = datetime.fromisoformat(end_date_str)

        # Get calendars to analyze
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
            return None

        # Collect all events in date range
        all_events = []
        calendar_stats = defaultdict(int)

        for calendar in calendars:
            events = calendar.events()

            for event in events:
                event_start = event.start_date()

                # Check if event falls within date range
                if start_date <= event_start <= end_date:
                    event_data = {
                        'title': event.summary() or 'Untitled',
                        'start': event_start,
                        'end': event.end_date(),
                        'calendar': calendar.name(),
                        'location': event.location() or '',
                        'duration_hours': (event.end_date() - event_start).total_seconds() / 3600
                    }
                    all_events.append(event_data)
                    calendar_stats[calendar.name()] += 1

        # Generate summary report
        total_events = len(all_events)
        total_duration = sum(event['duration_hours'] for event in all_events)

        # Group by day
        events_by_day = defaultdict(list)
        for event in all_events:
            day = event['start'].date()
            events_by_day[day].append(event)

        # Display summary
        print(f"Calendar Summary: {start_date.strftime('%Y-%m-%d')} to {end_date.strftime('%Y-%m-%d')}")
        print("=" * 60)
        print(f"Total Events: {total_events}")
        print(f"Total Duration: {total_duration:.1f} hours")
        print(f"Calendars: {', '.join(calendar_stats.keys())}")
        print()

        # Calendar breakdown
        print("Events by Calendar:")
        for cal_name, count in calendar_stats.items():
            print(f"  {cal_name}: {count} events")
        print()

        # Daily breakdown
        print("Daily Breakdown:")
        for day in sorted(events_by_day.keys()):
            day_events = events_by_day[day]
            day_duration = sum(e['duration_hours'] for e in day_events)
            print(f"  {day.strftime('%Y-%m-%d')}: {len(day_events)} events ({day_duration:.1f} hours)")
            for event in sorted(day_events, key=lambda x: x['start']):
                start_time = event['start'].strftime('%H:%M')
                print(f"    {start_time}: {event['title']}")
        print()

        # Top event titles
        title_counts = defaultdict(int)
        for event in all_events:
            title_counts[event['title']] += 1

        print("Most Common Event Titles:")
        for title, count in sorted(title_counts.items(), key=lambda x: x[1], reverse=True)[:5]:
            print(f"  {title}: {count} times")

        return {
            'total_events': total_events,
            'total_duration': total_duration,
            'events_by_calendar': dict(calendar_stats),
            'events_by_day': dict(events_by_day)
        }

    except Exception as e:
        print(f"Error generating calendar summary: {e}")
        return None

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python calendar_summary.py '2024-01-01' '2024-01-31' [--calendar 'Work']")
        sys.exit(1)

    start_date = sys.argv[1]
    end_date = sys.argv[2]
    calendar = None

    # Parse optional calendar argument
    for arg in sys.argv[3:]:
        if arg.startswith('--calendar'):
            calendar = arg.split('=')[1] if '=' in arg else sys.argv[sys.argv.index(arg) + 1]

    summary = generate_calendar_summary(start_date, end_date, calendar)
    sys.exit(0 if summary else 1)