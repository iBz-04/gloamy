# EventKit query example (JXA + ObjC)

```javascript
ObjC.import('EventKit');
ObjC.import('Foundation');

const store = $.EKEventStore.alloc.init;
let granted = false;
const group = $.dispatch_group_create();
$.dispatch_group_enter(group);
store.requestAccessToEntityTypeCompletion($.EKEntityTypeEvent, (ok, err) => {
  granted = ok;
  $.dispatch_group_leave(group);
});
$.dispatch_group_wait(group, $.DISPATCH_TIME_FOREVER);
if (!granted) throw new Error("Calendar access denied");

// Date range
const now = $.NSDate.date;
const end = $.NSDate.dateWithTimeIntervalSinceNow(30 * 24 * 3600);

// Pick calendar by title
const cals = store.calendarsForEntityType($.EKEntityTypeEvent);
let target = null;
for (let i = 0; i < cals.count; i++) {
  const c = cals.objectAtIndex(i);
  if (c.title.js === "Work") { target = c; break; }
}
if (!target) throw new Error("Calendar 'Work' not found");

// Predicate + fetch
const predicate = store.predicateForEventsWithStartDateEndDateCalendars(now, end, [target]);
const events = store.eventsMatchingPredicate(predicate);

const result = [];
for (let i = 0; i < events.count; i++) {
  const e = events.objectAtIndex(i);
  result.push({
    title: e.title.js,
    start: e.startDate.js,
    id: e.eventIdentifier.js
  });
}
console.log(JSON.stringify(result, null, 2));
```

