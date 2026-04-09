# PyXA Calendar Module API Reference

> **PyXA Module** - Control macOS Calendar.app using JXA-like syntax from Python.

This reference documents all classes, methods, properties, and enums in the PyXA Calendar module. For practical examples and usage patterns, see [calendar-basics.md](calendar-basics.md) and [calendar-recipes.md](calendar-recipes.md).

## Contents

- [Class Hierarchy](#class-hierarchy)
- [XACalendarApplication](#xacalendarapplication)
- [XACalendarCalendar](#xacalendarcalendar)
- [XACalendarEvent](#xacalendarevent)
- [XACalendarAttendee](#xacalendarattendee)
- [XACalendarAttachment](#xacalendarattachment)
- [XACalendarAlarm](#xacalendaralarm)
- [XACalendarDocument](#xacalendardocument)
- [XACalendarWindow](#xacalendarwindow)
- [List Classes](#list-classes)
- [Enumerations](#enumerations)
- [Quick Reference Tables](#quick-reference-tables)

---

## Class Hierarchy

```
XAObject
├── XACalendarApplication (XASBApplication, XACanOpenPath)
│   ├── XACalendarCalendar
│   │   └── XACalendarEvent
│   │       ├── XACalendarAttendee
│   │       └── XACalendarAttachment
│   ├── XACalendarDocument
│   └── XACalendarWindow (XASBWindow)
├── XACalendarAlarm
│   ├── XACalendarDisplayAlarm
│   ├── XACalendarMailAlarm
│   ├── XACalendarOpenFileAlarm
│   └── XACalendarSoundAlarm
└── List Classes
    ├── XACalendarCalendarList
    ├── XACalendarEventList
    ├── XACalendarAttendeeList
    ├── XACalendarAttachmentList
    └── XACalendarDocumentList
```

---

## XACalendarApplication

**Bases:** `XASBApplication`, `XACanOpenPath`

Main entry point for interacting with Calendar.app.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `default_calendar` | `XACalendarCalendar` | The default calendar for new events |
| `frontmost` | `bool` | Whether Calendar is the frontmost application |
| `name` | `str` | Application name ("Calendar") |
| `version` | `str` | Application version |
| `properties` | `dict` | All properties of the Calendar application |

### Methods

#### `calendars(filter=None) -> XACalendarCalendarList`

Returns a list of calendars matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
import PyXA
calendar = PyXA.Application("Calendar")
calendars = calendar.calendars()
for cal in calendars:
    print(cal.name)
```

#### `documents(filter=None) -> XACalendarDocumentList`

Returns a list of open documents matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

#### `new_calendar(name='New Calendar') -> XACalendarCalendar`

Creates a new calendar with the specified name.

**Parameters:**
- `name` (`str`) - Name for the new calendar (default: "New Calendar")

**Returns:** The newly created calendar

**Example:**
```python
calendar = PyXA.Application("Calendar")
work_cal = calendar.new_calendar("Work Events")
print(f"Created: {work_cal.name}")
```

#### `new_event(summary, start_date, end_date, calendar=None) -> XACalendarEvent`

Creates a new event.

**Parameters:**
- `summary` (`str`) - Event title/summary
- `start_date` (`datetime`) - Event start time
- `end_date` (`datetime`) - Event end time
- `calendar` (`XACalendarCalendar | None`) - Calendar to add event to (default: default calendar)

**Returns:** The newly created event

**Example:**
```python
from datetime import datetime, timedelta

calendar = PyXA.Application("Calendar")
event = calendar.new_event(
    summary="Team Meeting",
    start_date=datetime.now(),
    end_date=datetime.now() + timedelta(hours=1)
)
print(f"Created event: {event.summary}")
```

#### `make(specifier, properties=None, data=None) -> XAObject`

Creates a new element without adding to any list. Use `XAList.push()` to add.

**Parameters:**
- `specifier` (`str | ObjectType`) - Class name to create
- `properties` (`dict`) - Properties for the object
- `data` (`Any`) - Initialization data

#### `reload_calendars() -> XACalendarApplication`

Reloads calendars from all sources. Returns self for chaining.

#### `subscribe_to(url) -> XACalendarCalendar`

Subscribes to a calendar at the specified URL.

**Parameters:**
- `url` (`str`) - URL of the calendar to subscribe to (iCal format)

**Returns:** The subscribed calendar

**Example:**
```python
calendar = PyXA.Application("Calendar")
subscribed = calendar.subscribe_to("https://example.com/calendar.ics")
```

#### `switch_view_to(view) -> XACalendarApplication`

Switches the calendar view.

**Parameters:**
- `view` (`ViewType`) - View to switch to (DAY, WEEK, MONTH, YEAR)

**Returns:** Self for method chaining

**Example:**
```python
calendar = PyXA.Application("Calendar")
calendar.switch_view_to(XACalendarApplication.ViewType.WEEK)
```

#### `view_calendar_at(date, view=None) -> XACalendarApplication`

Navigates to a specific date, optionally changing view.

**Parameters:**
- `date` (`datetime`) - Date to navigate to
- `view` (`ViewType | None`) - Optional view to switch to

**Returns:** Self for method chaining

**Example:**
```python
from datetime import datetime

calendar = PyXA.Application("Calendar")
# View January 1st in month view
calendar.view_calendar_at(
    datetime(2025, 1, 1),
    XACalendarApplication.ViewType.MONTH
)
```

---

## XACalendarCalendar

**Bases:** `XAObject`

Represents a calendar in Calendar.app.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Calendar name |
| `description` | `str` | Calendar description |
| `color` | `XAColor` | Calendar display color |
| `writable` | `bool` | Whether the calendar can be modified |
| `calendar_obj` | `EKCalendar` | Underlying EventKit calendar object |
| `properties` | `dict` | All calendar properties |

### Methods

#### `events(filter=None) -> XACalendarEventList`

Returns events in this calendar matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
calendar = PyXA.Application("Calendar")
work_cal = calendar.calendars().by_name("Work")
all_events = work_cal.events()
```

#### `events_in_range(start_date, end_date) -> XACalendarEventList`

Returns events within a date range.

**Parameters:**
- `start_date` (`datetime`) - Range start
- `end_date` (`datetime`) - Range end

**Example:**
```python
from datetime import datetime, timedelta

calendar = PyXA.Application("Calendar")
cal = calendar.calendars()[0]

# Get events for next week
start = datetime.now()
end = start + timedelta(days=7)
events = cal.events_in_range(start, end)

for event in events:
    print(f"{event.summary}: {event.start_date}")
```

#### `events_today() -> XACalendarEventList`

Returns all events occurring today.

**Example:**
```python
calendar = PyXA.Application("Calendar")
cal = calendar.calendars()[0]
today_events = cal.events_today()
print(f"You have {len(today_events)} events today")
```

#### `new_event(name, start_date, end_date) -> XACalendarEvent`

Creates a new event in this calendar.

**Parameters:**
- `name` (`str`) - Event title
- `start_date` (`datetime`) - Event start time
- `end_date` (`datetime`) - Event end time

**Returns:** The newly created event

**Example:**
```python
from datetime import datetime, timedelta

calendar = PyXA.Application("Calendar")
work_cal = calendar.calendars().by_name("Work")

event = work_cal.new_event(
    "Project Review",
    datetime(2025, 1, 15, 14, 0),
    datetime(2025, 1, 15, 15, 0)
)
```

#### `delete() -> XACalendarEvent`

Deletes this calendar.

---

## XACalendarEvent

**Bases:** `XAObject`

Represents a calendar event.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `summary` | `str` | Event title/summary |
| `description` | `str` | Event description/notes |
| `start_date` | `datetime` | Event start date and time |
| `end_date` | `datetime` | Event end date and time |
| `allday_event` | `bool` | Whether this is an all-day event |
| `location` | `str` | Event location |
| `url` | `str` | Associated URL |
| `recurrence` | `str` | Recurrence rule string |
| `status` | `EventStatus` | Event status (CONFIRMED, TENTATIVE, CANCELLED, NONE) |
| `sequence` | `int` | Modification sequence number |
| `stamp_date` | `datetime` | Last modification timestamp |
| `excluded_dates` | `list[datetime]` | Dates excluded from recurrence |
| `uid` | `str` | Unique event identifier |
| `xa_event_obj` | `type` | Underlying EventKit event object |
| `properties` | `dict` | All event properties |

### Methods

#### `attendees(filter=None) -> XACalendarAttendeeList`

Returns attendees for this event.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
calendar = PyXA.Application("Calendar")
event = calendar.calendars()[0].events()[0]
attendees = event.attendees()
for attendee in attendees:
    print(f"{attendee.display_name}: {attendee.email}")
```

#### `attachments(filter=None) -> XACalendarAttachmentList`

Returns attachments for this event.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

#### `add_attachment(path) -> XACalendarEvent`

Adds an attachment to this event.

**Parameters:**
- `path` (`str`) - Path to the file to attach

**Returns:** Self for method chaining

**Example:**
```python
event = calendar.calendars()[0].events()[0]
event.add_attachment("/Users/me/Documents/agenda.pdf")
```

#### `delete() -> None`

Deletes this event.

#### `duplicate() -> XACalendarEvent`

Creates a copy of this event in the same calendar.

**Returns:** The duplicated event

#### `duplicate_to(calendar) -> XACalendarEvent`

Duplicates this event to another calendar.

**Parameters:**
- `calendar` (`XACalendarCalendar`) - Target calendar

**Returns:** The duplicated event

**Example:**
```python
calendar = PyXA.Application("Calendar")
work_cal = calendar.calendars().by_name("Work")
personal_cal = calendar.calendars().by_name("Personal")

# Copy event to personal calendar
event = work_cal.events()[0]
copied = event.duplicate_to(personal_cal)
```

#### `move_to(calendar) -> XACalendarEvent`

Moves this event to another calendar.

**Parameters:**
- `calendar` (`XACalendarCalendar`) - Target calendar

**Returns:** Self for method chaining

#### `show() -> XACalendarEvent`

Opens the event in Calendar.app for viewing.

**Returns:** Self for method chaining

---

## XACalendarAttendee

**Bases:** `XAObject`

Represents an event attendee.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `display_name` | `str` | Attendee's display name |
| `email` | `str` | Attendee's email address |
| `participation_status` | `ParticipationStatus` | RSVP status (ACCEPTED, DECLINED, TENTATIVE, UNKNOWN) |
| `properties` | `dict` | All attendee properties |

---

## XACalendarAttachment

**Bases:** `XAObject`

Represents an event attachment.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `file` | `XAPath` | File path of attachment |
| `file_name` | `str` | Name of the attached file |
| `type` | `str` | MIME type of the attachment |
| `url` | `XAURL` | URL of the attachment (if URL-based) |
| `uuid` | `str` | Unique identifier |

### Methods

#### `open() -> XACalendarAttachment`

Opens the attachment in its default application.

**Returns:** Self for method chaining

---

## XACalendarAlarm

**Bases:** `XAObject`

Base class for all alarm types.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `trigger_date` | `datetime` | Absolute alarm trigger time |
| `trigger_interval` | `int` | Relative offset in seconds from event start |
| `properties` | `dict` | All alarm properties |

### XACalendarDisplayAlarm

**Bases:** `XACalendarAlarm`

A visual notification alarm.

### XACalendarMailAlarm

**Bases:** `XACalendarAlarm`

An email notification alarm.

### XACalendarOpenFileAlarm

**Bases:** `XACalendarAlarm`

An alarm that opens a file.

**Additional Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `file_path` | `str` | Path to file to open when alarm triggers |

### XACalendarSoundAlarm

**Bases:** `XACalendarAlarm`

An alarm that plays a sound.

**Additional Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `sound_file` | `str` | Path to sound file |
| `sound_name` | `str` | Name of system sound |

---

## XACalendarDocument

**Bases:** `XAObject`

Represents a Calendar document.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Document name |
| `file` | `XAPath` | Document file path |
| `modified` | `bool` | Whether document has unsaved changes |
| `properties` | `dict` | All document properties |

---

## XACalendarWindow

**Bases:** `XASBWindow`

Represents a Calendar window.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `document` | `XACalendarDocument` | Document displayed in window |
| `properties` | `dict` | All window properties |

---

## List Classes

PyXA provides list wrapper classes with fast enumeration and bulk property access.

### XACalendarCalendarList

Bulk methods for calendar lists:

```python
calendars = app.calendars()
calendars.name()          # -> list[str]
calendars.description()   # -> list[str]
calendars.color()         # -> list[XAColor]
calendars.writable()      # -> list[bool]
calendars.properties()    # -> list[dict]
```

Filter methods:
```python
calendars.by_name("Work")
calendars.by_writable(True)
calendars.by_description("Personal calendar")
```

### XACalendarEventList

Bulk methods for event lists:

```python
events = calendar.events()
events.summary()          # -> list[str]
events.description()      # -> list[str]
events.start_date()       # -> list[datetime]
events.end_date()         # -> list[datetime]
events.allday_event()     # -> list[bool]
events.location()         # -> list[str]
events.url()              # -> list[str]
events.recurrence()       # -> list[str]
events.status()           # -> list[EventStatus]
events.uid()              # -> list[str]
events.properties()       # -> list[dict]
```

Filter methods:
```python
events.by_summary("Meeting")
events.by_location("Conference Room")
events.by_allday_event(True)
events.by_status(XACalendarApplication.EventStatus.CONFIRMED)
```

### XACalendarAttendeeList

Bulk methods for attendee lists:

```python
attendees = event.attendees()
attendees.display_name()          # -> list[str]
attendees.email()                 # -> list[str]
attendees.participation_status()  # -> list[ParticipationStatus]
attendees.properties()            # -> list[dict]
```

Filter methods:
```python
attendees.by_email("user@example.com")
attendees.by_display_name("John Doe")
attendees.by_participation_status(XACalendarApplication.ParticipationStatus.ACCEPTED)
```

### XACalendarAttachmentList

Bulk methods for attachment lists:

```python
attachments = event.attachments()
attachments.file_name()    # -> list[str]
attachments.type()         # -> list[str]
attachments.uuid()         # -> list[str]
attachments.properties()   # -> list[dict]
```

Filter methods:
```python
attachments.by_file_name("document.pdf")
attachments.by_type("application/pdf")
```

### XACalendarDocumentList

Bulk methods for document lists:

```python
docs = app.documents()
docs.name()        # -> list[str]
docs.modified()    # -> list[bool]
docs.file()        # -> list[XAPath]
docs.properties()  # -> list[dict]
```

Filter methods:
```python
docs.by_name("Calendar")
docs.by_modified(True)
```

---

## Enumerations

### EventStatus

Event confirmation status.

| Value | Description |
|-------|-------------|
| `NONE` | No status set |
| `CONFIRMED` | Event is confirmed |
| `TENTATIVE` | Event is tentative |
| `CANCELLED` | Event is cancelled |

**Usage:**
```python
from PyXA import XACalendarApplication

# Filter confirmed events
confirmed = events.by_status(XACalendarApplication.EventStatus.CONFIRMED)

# Check event status
if event.status == XACalendarApplication.EventStatus.CANCELLED:
    print("Event was cancelled")
```

### ParticipationStatus

Attendee RSVP status.

| Value | Description |
|-------|-------------|
| `UNKNOWN` | Status not known |
| `ACCEPTED` | Attendee accepted |
| `DECLINED` | Attendee declined |
| `TENTATIVE` | Attendee tentatively accepted |

**Usage:**
```python
# Get accepted attendees
accepted = attendees.by_participation_status(
    XACalendarApplication.ParticipationStatus.ACCEPTED
)
```

### Priority

Event priority levels.

| Value | Description |
|-------|-------------|
| `NONE` | No priority set |
| `LOW` | Low priority |
| `MEDIUM` | Medium priority |
| `HIGH` | High priority |

### ViewType

Calendar view options.

| Value | Description |
|-------|-------------|
| `DAY` | Day view |
| `WEEK` | Week view |
| `MONTH` | Month view |
| `YEAR` | Year view |

**Usage:**
```python
# Switch to week view
calendar.switch_view_to(XACalendarApplication.ViewType.WEEK)

# Navigate to specific date in month view
calendar.view_calendar_at(
    datetime(2025, 6, 1),
    XACalendarApplication.ViewType.MONTH
)
```

### ObjectType

Creatable object types for the `make()` method.

| Value | Description |
|-------|-------------|
| `CALENDAR` | Calendar |
| `EVENT` | Event |
| `DOCUMENT` | Document |
| `DISPLAY_ALARM` | Display notification alarm |
| `MAIL_ALARM` | Email alarm |
| `OPEN_FILE_ALARM` | Open file alarm |
| `SOUND_ALARM` | Sound alarm |

**Usage:**
```python
# Create an event using make()
event = calendar.make(
    XACalendarApplication.ObjectType.EVENT,
    properties={
        "summary": "New Meeting",
        "start_date": datetime.now(),
        "end_date": datetime.now() + timedelta(hours=1)
    }
)
```

---

## Quick Reference Tables

### Common Operations

| Task | Code |
|------|------|
| Get Calendar app | `calendar = PyXA.Application("Calendar")` |
| Get all calendars | `calendars = calendar.calendars()` |
| Get calendar by name | `cal = calendar.calendars().by_name("Work")` |
| Get default calendar | `default = calendar.default_calendar` |
| Create new calendar | `cal = calendar.new_calendar("My Calendar")` |
| Create event | `event = calendar.new_event("Meeting", start, end)` |
| Get today's events | `events = cal.events_today()` |
| Get events in range | `events = cal.events_in_range(start, end)` |
| Get event attendees | `attendees = event.attendees()` |
| Move event | `event.move_to(other_calendar)` |
| Duplicate event | `copy = event.duplicate_to(other_calendar)` |
| Delete event | `event.delete()` |
| Switch view | `calendar.switch_view_to(ViewType.WEEK)` |
| Subscribe to calendar | `cal = calendar.subscribe_to(url)` |
| Reload calendars | `calendar.reload_calendars()` |

### Property Access Patterns

```python
# Single object property access
event = cal.events()[0]
print(event.summary)
print(event.start_date)
print(event.location)

# Bulk access on lists (returns list)
events = cal.events()
print(events.summary())      # -> list[str]
print(events.start_date())   # -> list[datetime]
print(events.location())     # -> list[str]

# Filtering
meetings = events.by_summary("Meeting")
all_day = events.by_allday_event(True)
```

### Date Range Queries

```python
from datetime import datetime, timedelta

# Today's events
today = cal.events_today()

# This week
start = datetime.now()
end = start + timedelta(days=7)
this_week = cal.events_in_range(start, end)

# Specific month
jan_start = datetime(2025, 1, 1)
jan_end = datetime(2025, 1, 31, 23, 59, 59)
january_events = cal.events_in_range(jan_start, jan_end)
```

### Creating Events with Properties

```python
from datetime import datetime, timedelta

# Basic event
event = cal.new_event(
    "Team Standup",
    datetime(2025, 1, 15, 9, 0),
    datetime(2025, 1, 15, 9, 30)
)

# Event via application with calendar selection
event = calendar.new_event(
    summary="Quarterly Review",
    start_date=datetime(2025, 3, 15, 14, 0),
    end_date=datetime(2025, 3, 15, 16, 0),
    calendar=calendar.calendars().by_name("Work")
)
```

---

## See Also

- [PyXA Calendar Documentation](https://skaplanofficial.github.io/PyXA/reference/apps/calendar.html) - Official PyXA documentation
- [calendar-basics.md](calendar-basics.md) - JXA fundamentals for Calendar
- [calendar-recipes.md](calendar-recipes.md) - Common automation patterns
- [calendar-advanced.md](calendar-advanced.md) - EventKit bridge and advanced patterns
- [eventkit-query.md](eventkit-query.md) - EventKit query examples
