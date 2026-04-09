# Reminders JXA Basics

## Initialize
```javascript
const app = Application("Reminders");
app.includeStandardAdditions = true; // optional UI/file helpers
```

## Specifier vs data
- `app.lists` → specifier.
- `app.lists.name()` → array of list names (one IPC).
- Access by name/id: `app.lists.byName("Inbox")`, `app.lists.byId("...")`.
- Reminder specifier: `app.lists.byName("Inbox").reminders`.

## Reading
- Batch read: `list.reminders.name()`; `list.reminders.id()`.
- Filter server-side: `list.reminders.whose({ completed: false })`.
- Date compare: `reminders.whose({ dueDate: { _lessThan: new Date() }})`.
- Operators: `_equals`, `_notEquals`, `_greaterThan`, `_lessThan`, `_beginsWith`, `_endsWith`, `_contains`, `_and`, `_or`.

## Writing
- Assign on specifier updates all matches: `reminders.priority = 1`.
- Use snapshots when modifying filter criteria: fetch `ids = spec.id()` then loop `byId`.

## Creating (stable pattern)
```javascript
const target = app.lists.byName("Inbox");
const r = app.Reminder({
  name: "Draft review",
  body: "Include metrics",
  priority: 1,
  dueDate: new Date()
});
target.reminders.push(r); // commit
```
- Avoid `make`; `.push` is more reliable.

## Moving
- No move command; implement copy-delete.
