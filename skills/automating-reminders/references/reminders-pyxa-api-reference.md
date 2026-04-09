# PyXA Reminders Module API Reference

> **New in PyXA version 0.0.1** - Control macOS Reminders using JXA-like syntax from Python.

This reference documents all classes, methods, properties, and enums in the PyXA Reminders module. For practical examples and usage patterns, see [reminders-recipes.md](reminders-recipes.md).

## Contents

- [Class Hierarchy](#class-hierarchy)
- [XARemindersApplication](#xaremindersapplication)
- [XARemindersList](#xareminderslist)
- [XARemindersReminder](#xaremindersreminder)
- [XARemindersAlarm](#xaremindersalarm)
- [XARemindersRecurrenceRule](#xaremindersrecurrencerule)
- [XARemindersAccount](#xaremindersaccount)
- [XARemindersDocument](#xaremindersdocument)
- [XARemindersWindow](#xareminderswindow)
- [List Classes](#list-classes)
- [Enumerations](#enumerations)
- [Quick Reference Tables](#quick-reference-tables)

---

## Class Hierarchy

```
XAObject
├── XARemindersApplication
│   ├── XARemindersAccount
│   ├── XARemindersList
│   │   └── XARemindersReminder
│   │       ├── XARemindersAlarm
│   │       └── XARemindersRecurrenceRule
│   ├── XARemindersDocument
│   └── XARemindersWindow
└── List Classes
    ├── XARemindersAccountList
    ├── XARemindersListList
    ├── XARemindersReminderList
    ├── XARemindersAlarmList
    └── XARemindersDocumentList
```

---

## XARemindersApplication

**Bases:** `XAApplication`

Main entry point for interacting with Reminders.app.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `default_account` | `XARemindersAccount` | The default Reminders account |
| `default_list` | `XARemindersList` | The default reminder list |
| `frontmost` | `bool` | Whether Reminders is frontmost application |
| `name` | `str` | Application name ("Reminders") |
| `version` | `str` | Application version string |

### Methods

#### `accounts(filter=None) -> XARemindersAccountList`

Returns a list of Reminders accounts matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
import PyXA
app = PyXA.Application("Reminders")
accounts = app.accounts()
for account in accounts:
    print(account.name)
```

#### `documents(filter=None) -> XARemindersDocumentList`

Returns a list of open documents matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

#### `lists(filter=None) -> XARemindersListList`

Returns reminder lists matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
all_lists = app.lists()
inbox = app.lists().by_name("Inbox")
```

#### `reminders(filter=None) -> XARemindersReminderList`

Returns all reminders across all lists matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
all_reminders = app.reminders()
high_priority = app.reminders().by_priority(1)
```

#### `new_reminder(name='New Reminder', due_date=None, reminder_list=None) -> XARemindersReminder`

Creates a new reminder.

**Parameters:**
- `name` (`str`) - Name/title of the reminder
- `due_date` (`datetime | None`) - Due date for the reminder
- `reminder_list` (`XARemindersList | None`) - List to add reminder to (uses default if None)

**Returns:** The newly created reminder

**Example:**
```python
from datetime import datetime, timedelta
reminder = app.new_reminder(
    name="Call dentist",
    due_date=datetime.now() + timedelta(days=1),
    reminder_list=app.lists().by_name("Personal")
)
```

#### `new_list(name='New List', color='#FF0000', emblem='symbol0') -> XARemindersList`

Creates a new reminder list.

**Parameters:**
- `name` (`str`) - Name of the list
- `color` (`str`) - Hex color code for the list
- `emblem` (`str`) - Icon/emblem name for the list

**Returns:** The newly created list

**Example:**
```python
work_list = app.new_list(
    name="Work Tasks",
    color="#0000FF",
    emblem="symbol1"
)
```

#### `make(specifier, properties=None, data=None)`

Creates a new element without adding to any list. Use `XAList.push()` to add.

**Parameters:**
- `specifier` (`str | ObjectType`) - Class name to create ('reminder', 'list')
- `properties` (`dict`) - Properties for the object
- `data` (`Any`) - Initialization data

---

## XARemindersList

**Bases:** `XAObject`

Represents a reminder list with organizational features.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier for the list |
| `name` | `str` | Name of the list |
| `color` | `str` | Hex color code |
| `emblem` | `str` | Icon/emblem name |
| `container` | `XARemindersAccount | XARemindersList` | Parent container |
| `sharing_status` | `bool` | Whether the list is shared |
| `sharees` | `list` | List of people the list is shared with |
| `subscription_url` | `str` | URL for subscribing to the list |
| `summary` | `str` | Summary description |
| `properties` | `dict` | All list properties |

### Methods

#### `reminders(filter=None) -> XARemindersReminderList`

Returns reminders in this list matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
inbox = app.lists().by_name("Inbox")
incomplete = inbox.reminders().by_completed(False)
```

#### `show() -> XARemindersList`

Shows the list in the Reminders application.

**Returns:** Self for method chaining

#### `delete() -> None`

Deletes the list and all its reminders.

**Warning:** This action is irreversible.

---

## XARemindersReminder

**Bases:** `XAObject`

Represents an individual reminder item with full task management capabilities.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `name` | `str` | Title of the reminder |
| `body` | `str` | Notes/body text |
| `notes` | `str` | Additional user notes |
| `due_date` | `datetime | None` | Due date and time |
| `allday_due_date` | `datetime | None` | All-day due date |
| `all_day` | `bool` | Whether this is an all-day reminder |
| `completed` | `bool` | Whether the reminder is completed |
| `completion_date` | `datetime | None` | When the reminder was completed |
| `flagged` | `bool` | Whether the reminder is flagged |
| `priority` | `int` | Priority level (see table below) |
| `creation_date` | `datetime` | When the reminder was created |
| `modification_date` | `datetime` | When the reminder was last modified |
| `remind_me_date` | `datetime | None` | Alert/notification date |
| `container` | `XARemindersList | XARemindersReminder` | Parent list or reminder |
| `recurrence_rule` | `XARemindersRecurrenceRule` | Recurrence settings |
| `url` | `XAURL` | Associated URL |
| `properties` | `dict` | All reminder properties |

#### Priority Values

| Value | Meaning |
|-------|---------|
| `0` | None (no priority) |
| `1-4` | High priority |
| `5` | Medium priority |
| `6-9` | Low priority |

### Methods

#### `alarms(filter=None) -> XARemindersAlarmList`

Returns alarms associated with this reminder.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

#### `move_to(list: XARemindersList) -> XARemindersReminder`

Moves the reminder to a different list.

**Parameters:**
- `list` (`XARemindersList`) - Target list to move to

**Returns:** Self for method chaining

**Example:**
```python
reminder = inbox.reminders()[0]
reminder.move_to(app.lists().by_name("Work"))
```

#### `show() -> XARemindersReminder`

Shows the reminder in the Reminders application.

**Returns:** Self for method chaining

#### `delete() -> None`

Deletes the reminder.

---

## XARemindersAlarm

**Bases:** `XAObject`

Alarm configuration for reminders with date and location support.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `date` | `datetime` | Alarm trigger date/time |
| `location` | `XALocation` | Location for location-based alarm |
| `proximity_direction` | `str` | 'arriving' or 'departing' for location alarms |
| `snoozed` | `bool` | Whether the alarm has been snoozed |

### Methods

#### `set_date(date: datetime) -> None`

Sets the alarm trigger date.

**Parameters:**
- `date` (`datetime`) - Date and time to trigger the alarm

#### `set_location(location: XALocation) -> None`

Sets a location-based alarm.

**Parameters:**
- `location` (`XALocation`) - Location that triggers the alarm

---

## XARemindersRecurrenceRule

**Bases:** `XAObject`

Manages reminder repetition patterns.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `frequency` | `str` | 'daily', 'weekly', 'monthly', or 'yearly' |
| `interval` | `int` | Interval between occurrences |
| `end_date` | `datetime` | When recurrence ends |

### Methods

#### `set_frequency(frequency: Literal['daily', 'weekly', 'monthly', 'yearly']) -> None`

Sets the recurrence frequency.

**Parameters:**
- `frequency` (`str`) - One of 'daily', 'weekly', 'monthly', 'yearly'

#### `set_interval(interval: int) -> None`

Sets the interval between occurrences.

**Parameters:**
- `interval` (`int`) - Number of frequency units between occurrences

**Example:**
```python
# Every 2 weeks
rule.set_frequency('weekly')
rule.set_interval(2)
```

#### `set_end_date(end_date: datetime) -> None`

Sets when the recurrence should end.

**Parameters:**
- `end_date` (`datetime`) - Date to stop recurring

---

## XARemindersAccount

**Bases:** `XAObject`

Represents a Reminders account (iCloud, Exchange, etc.).

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `name` | `str` | Account name |

---

## XARemindersDocument

**Bases:** `XAObject`

File-based document handling for Reminders.

### Methods

#### `save() -> XARemindersDocument`

Saves the document.

**Returns:** Self for method chaining

#### `close(save=True) -> None`

Closes the document.

**Parameters:**
- `save` (`bool`) - Whether to save before closing

---

## XARemindersWindow

**Bases:** `XAObject`

Window management for Reminders application.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `document` | `XARemindersDocument` | Document displayed in window |

### Methods

#### `lists(filter=None) -> XARemindersListList`

Returns lists visible in this window.

#### `reminders(filter=None) -> XARemindersReminderList`

Returns reminders visible in this window.

#### `save() -> XARemindersWindow`

Saves the window's document.

#### `close(save=True) -> None`

Closes the window.

#### `print(properties, show_dialog=True) -> XARemindersWindow`

Prints the window content.

---

## List Classes

PyXA provides list wrapper classes with fast enumeration and bulk property access.

### XARemindersAccountList

Bulk methods for account lists:

```python
accounts = app.accounts()
accounts.id()       # -> list[str]
accounts.name()     # -> list[str]
```

Filter methods:
```python
accounts.by_id("account-id")
accounts.by_name("iCloud")
```

### XARemindersListList

Bulk methods for reminder list collections:

```python
lists = app.lists()
lists.id()              # -> list[str]
lists.name()            # -> list[str]
lists.color()           # -> list[str]
lists.emblem()          # -> list[str]
lists.sharing_status()  # -> list[bool]
lists.properties()      # -> list[dict]
```

Filter methods:
```python
lists.by_id("list-id")
lists.by_name("Work")
lists.by_color("#FF0000")
lists.by_sharing_status(True)
```

### XARemindersReminderList

Bulk methods for reminder collections:

```python
reminders = app.reminders()
reminders.id()               # -> list[str]
reminders.name()             # -> list[str]
reminders.body()             # -> list[str]
reminders.due_date()         # -> list[datetime | None]
reminders.completed()        # -> list[bool]
reminders.completion_date()  # -> list[datetime | None]
reminders.flagged()          # -> list[bool]
reminders.priority()         # -> list[int]
reminders.creation_date()    # -> list[datetime]
reminders.modification_date()  # -> list[datetime]
reminders.remind_me_date()   # -> list[datetime | None]
reminders.all_day()          # -> list[bool]
reminders.properties()       # -> list[dict]
```

Filter methods:
```python
reminders.by_id("reminder-id")
reminders.by_name("Call dentist")
reminders.by_completed(True)
reminders.by_completed(False)
reminders.by_flagged(True)
reminders.by_priority(1)  # High priority
reminders.by_all_day(True)
```

**Bulk deletion:**
```python
completed = app.reminders().by_completed(True)
completed.delete()  # Delete all completed reminders
```

### XARemindersAlarmList

Bulk methods for alarm collections:

```python
alarms = reminder.alarms()
alarms.id()                  # -> list[str]
alarms.date()                # -> list[datetime]
alarms.proximity_direction() # -> list[str]
alarms.snoozed()             # -> list[bool]
```

Filter methods:
```python
alarms.by_snoozed(True)
alarms.by_proximity_direction("arriving")
```

---

## Enumerations

### ObjectType

Creatable object types for `make()` method.

| Value | Description |
|-------|-------------|
| `DOCUMENT` | Document object |
| `LIST` | Reminder list |
| `REMINDER` | Individual reminder |

**Usage:**
```python
from PyXA.apps.Reminders import XARemindersApplication

# Create using ObjectType enum
reminder = app.make(
    XARemindersApplication.ObjectType.REMINDER,
    properties={"name": "New Task", "priority": 1}
)
```

### Priority Constants

While not a formal enum, priority uses integer values:

| Value | Meaning | Usage |
|-------|---------|-------|
| `0` | None | No priority set |
| `1` | High (highest) | `priority=1` |
| `2-4` | High | Alternative high values |
| `5` | Medium | `priority=5` |
| `6-9` | Low | `priority=9` for lowest |

### RecurrenceFrequency (String Literals)

| Value | Description |
|-------|-------------|
| `'daily'` | Repeats every day |
| `'weekly'` | Repeats every week |
| `'monthly'` | Repeats every month |
| `'yearly'` | Repeats every year |

### ProximityDirection (String Literals)

For location-based alarms:

| Value | Description |
|-------|-------------|
| `'arriving'` | Trigger when arriving at location |
| `'departing'` | Trigger when leaving location |

---

## Quick Reference Tables

### Common Operations

| Task | Code |
|------|------|
| Get Reminders app | `app = PyXA.Application("Reminders")` |
| Get all lists | `lists = app.lists()` |
| Get list by name | `inbox = app.lists().by_name("Inbox")` |
| Get default list | `default = app.default_list` |
| Create new list | `app.new_list("Work", color="#0000FF")` |
| Get all reminders | `reminders = app.reminders()` |
| Get reminders in list | `reminders = inbox.reminders()` |
| Create reminder | `app.new_reminder("Task", due_date=date)` |
| Mark completed | `reminder.completed = True` |
| Set priority | `reminder.priority = 1` |
| Delete reminder | `reminder.delete()` |
| Move reminder | `reminder.move_to(other_list)` |

### Filtering Patterns

```python
# Get incomplete reminders
incomplete = app.reminders().by_completed(False)

# Get high priority reminders
urgent = app.reminders().by_priority(1)

# Get flagged reminders
flagged = app.reminders().by_flagged(True)

# Get reminders from specific list
work_tasks = app.lists().by_name("Work").reminders()

# Chain filters (get incomplete flagged items)
inbox = app.lists().by_name("Inbox")
important = inbox.reminders().by_completed(False)
flagged_important = [r for r in important if r.flagged]
```

### Date Handling

```python
from datetime import datetime, timedelta

# Due tomorrow
due_date = datetime.now() + timedelta(days=1)

# Remind 1 hour before due
remind_date = due_date - timedelta(hours=1)

# Create with dates
reminder = app.new_reminder(
    name="Meeting prep",
    due_date=due_date,
    reminder_list=app.default_list
)
reminder.remind_me_date = remind_date
```

### Property Access Patterns

```python
# Single object
reminder = app.reminders()[0]
print(reminder.name)
print(reminder.due_date)
print(reminder.completed)

# Bulk access on lists
reminders = app.reminders()
print(reminders.name())       # Returns list[str]
print(reminders.due_date())   # Returns list[datetime | None]
print(reminders.priority())   # Returns list[int]

# Filtering
incomplete = reminders.by_completed(False)
high_priority = reminders.by_priority(1)
```

---

## See Also

- [PyXA Reminders Documentation](https://skaplanofficial.github.io/PyXA/reference/apps/reminders.html) - Official PyXA documentation
- [reminders-recipes.md](reminders-recipes.md) - Practical usage examples
- [reminders-basics.md](reminders-basics.md) - JXA fundamentals
- [reminders-advanced.md](reminders-advanced.md) - Complex automation patterns
