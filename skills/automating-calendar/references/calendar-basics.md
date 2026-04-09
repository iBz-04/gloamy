# Calendar JXA basics

## Bootstrapping
```javascript
const App = Application.currentApplication();
App.includeStandardAdditions = true;
const Cal = Application("Calendar");
```

## List calendars
```javascript
const names = Cal.calendars.name();
```

## Create simple event
```javascript
const cal = Cal.calendars.byName("Work");
const start = new Date();
const end = new Date(start.getTime() + 60 * 60000);
cal.events.push(Cal.Event({ summary: "Meeting", startDate: start, endDate: end }));
```

