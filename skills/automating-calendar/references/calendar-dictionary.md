# Calendar dictionary translation table

```
AppleScript                         JXA
----------------------------------- ------------------------------------------
calendar "Work"                    Cal.calendars.byName("Work")
events of calendar                 Cal.calendars.events
summary of event                   evt.summary()
start date of event                evt.startDate()
make new event                     cal.events.push(Cal.Event({...}))
```

Notes:
- Collections are specifiers; call methods to read values.
- Use recurrence strings for RRULEs.

