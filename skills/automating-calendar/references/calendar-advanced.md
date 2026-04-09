# Calendar JXA advanced patterns

## Batch read warnings
- Avoid heavy `.whose()` on events; it is slow/unreliable.
- Prefer EventKit bridge for large queries.

## Time zones
- JS Date is local; to target another TZ, adjust offset manually or use EventKit.

## EventKit bridge (outline)
```javascript
ObjC.import('EventKit');
const store = $.EKEventStore.alloc.init;
// Request access and wait
// Build predicate and fetch events via eventsMatchingPredicate
```

## UI scripting (view changes)
```javascript
const se = Application("System Events");
const calProc = se.processes.byName("Calendar");
calProc.menuBars.menuBarItems.byName("View").menus.menuItems.byName("Day").click();
```

