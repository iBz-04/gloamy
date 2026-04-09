# EventKit time zone example

```javascript
ObjC.import('EventKit');
ObjC.import('Foundation');

const store = $.EKEventStore.alloc.init;
let granted = false;
const g = $.dispatch_group_create();
$.dispatch_group_enter(g);
store.requestAccessToEntityTypeCompletion($.EKEntityTypeEvent, (ok, err) => { granted = ok; $.dispatch_group_leave(g); });
$.dispatch_group_wait(g, $.DISPATCH_TIME_FOREVER);
if (!granted) throw new Error("Calendar access denied");

const cals = store.calendarsForEntityType($.EKEntityTypeEvent);
let target = null;
for (let i = 0; i < cals.count; i++) {
  const c = cals.objectAtIndex(i);
  if (c.title.js === "Work") { target = c; break; }
}
if (!target) throw new Error("Calendar 'Work' not found");

// Time zone
const tz = $.NSTimeZone.timeZoneWithName("America/New_York");

const evt = $.EKEvent.eventWithEventStore(store);
evt.title = "TZ Event";
evt.timeZone = tz;

// Build start/end using NSDateComponents (safer for TZ)
const cal = $.NSCalendar.currentCalendar;
const comps = $.NSDateComponents.alloc.init;
comps.year = 2026; comps.month = 1; comps.day = 15; comps.hour = 9; comps.minute = 0;
const start = cal.dateFromComponents(comps);
comps.hour = 10; // end time
const end = cal.dateFromComponents(comps);

evt.startDate = start;
evt.endDate = end;
evt.calendar = target;

const err = $();
const ok = store.saveEventSpanCommitError(evt, $.EKSpanThisEvent, true, err);
if (!ok) console.log("Save failed: " + err.localizedDescription.js);
```

