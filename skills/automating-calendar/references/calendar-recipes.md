# Calendar JXA recipes

## Create recurring event
```javascript
const cal = Cal.calendars.byName("Work");
const start = new Date();
const end = new Date(start.getTime() + 30 * 60000);
cal.events.push(Cal.Event({
  summary: "Daily standup",
  startDate: start,
  endDate: end,
  recurrence: "FREQ=DAILY;INTERVAL=1"
}));
```

## Add alarm
```javascript
const evt = cal.events[0];
const alarm = Cal.DisplayAlarm({ triggerInterval: -900 }); // 15 min
evt.displayAlarms.push(alarm);
```

## Add multiple alerts (1d, 1h, 15m)
```javascript
const evt = cal.events[0];
evt.displayAlarms.push(Cal.DisplayAlarm({ triggerInterval: -86400 })); // 1 day
evt.displayAlarms.push(Cal.DisplayAlarm({ triggerInterval: -3600 }));  // 1 hour
evt.displayAlarms.push(Cal.DisplayAlarm({ triggerInterval: -900 }));   // 15 min
```

## Move event (delete + recreate pattern)
```javascript
const evt = cal.events[0];
const props = {
  summary: evt.summary(),
  startDate: new Date(Date.now() + 3600 * 1000),
  endDate: new Date(Date.now() + 7200 * 1000)
};
cal.events.push(Cal.Event(props));
evt.delete();
```
