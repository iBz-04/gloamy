# JXA Calendar and Notes recipes

## List upcoming calendar events
```javascript
const calendar = Application("Calendar");
const cal = calendar.calendars.byName("Work");
const today = new Date();
const future = new Date();
future.setDate(future.getDate() + 7);
const events = cal.events.whose({ startDate: { ">": today, "<": future } });
const out = events.map(e => ({ title: e.title(), start: e.startDate().toISOString() }));
JSON.stringify({ count: out.length, events: out });
```

## Create a note
```javascript
const notes = Application("Notes");
const folder = notes.folders.byName("Notes");
const note = notes.Note({ name: "New Note", body: "Hello" });
folder.notes.push(note);
JSON.stringify({ success: true });
```
