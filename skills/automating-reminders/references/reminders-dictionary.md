# Reminders Dictionary & Types

## Core objects
- `Application("Reminders")`
- `lists` (element) → `List`
- `reminders` (element of list or app) → `Reminder`

## List
- Properties: `name`, `id`, `defaultList` (app only)
- Elements: `reminders`

## Reminder
- `name` (string, R/W)
- `body` (string, R/W)
- `completed` (boolean, R/W)
- `completionDate` (date, R/W)
- `creationDate` (date, R/O)
- `dueDate` (date, R/W)
- `remindMeDate` (date, R/W)
- `priority` (integer, R/W; 0/1/5/9)
- `id` (string URL, R/O)
- `container` (list, R/O)

## Commands
- Create: prefer `app.Reminder({...})` + `list.reminders.push(obj)` (instead of `make`).
- Delete: `app.delete(specifier)` (batch friendly).
- No move: use copy-delete.

## Filters
- `.whose` on collections with operators: `_equals`, `_notEquals`, `_greaterThan`, `_lessThan`, `_beginsWith`, `_endsWith`, `_contains`, `_and`, `_or`.
- Access properties on specifiers to batch read/write: `.name()`, `.id()`, `.priority = 1`.
