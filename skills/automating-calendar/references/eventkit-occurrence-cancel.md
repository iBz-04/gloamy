# EventKit: cancel a single occurrence

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

function findCalendar(title) {
  const cals = store.calendarsForEntityType($.EKEntityTypeEvent);
  for (let i = 0; i < cals.count; i++) {
    const c = cals.objectAtIndex(i);
    if (c.title.js === title) return c;
  }
  return null;
}

const targetCal = findCalendar("Work");
if (!targetCal) throw new Error("Calendar not found");

// Define the date window around the occurrence you want to cancel
const cal = $.NSCalendar.currentCalendar;
const startWindow = $.NSDate.dateWithTimeIntervalSinceNow(0); // now
const endWindow = $.NSDate.dateWithTimeIntervalSinceNow(7 * 24 * 3600); // next 7 days

const predicate = store.predicateForEventsWithStartDateEndDateCalendars(startWindow, endWindow, [targetCal]);
const events = store.eventsMatchingPredicate(predicate);

// Find the occurrence by title and date match
let targetOcc = null;
for (let i = 0; i < events.count; i++) {
  const e = events.objectAtIndex(i);
  if (e.title.js === "Weekly Sync" && e.startDate.timeIntervalSinceNow() > 0) {
    targetOcc = e; break;
  }
}

if (targetOcc) {
  const err = $();
  const ok = store.removeEventSpanCommitError(targetOcc, $.EKSpanThisEvent, true, err);
  if (!ok) console.log("Remove failed: " + err.localizedDescription.js);
}
```

Notes:
- The occurrence is identified by title and date window; refine matching as needed.
- `EKSpanThisEvent` removes only this occurrence; the series remains.

