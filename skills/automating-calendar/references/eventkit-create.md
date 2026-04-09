# EventKit create event with recurrence (JXA + ObjC)

```javascript
ObjC.import('EventKit');
ObjC.import('Foundation');

const store = $.EKEventStore.alloc.init;
let granted = false;
const g = $.dispatch_group_create();
$.dispatch_group_enter(g);
store.requestAccessToEntityTypeCompletion($.EKEntityTypeEvent, (ok, err) => {
  granted = ok;
  $.dispatch_group_leave(g);
});
$.dispatch_group_wait(g, $.DISPATCH_TIME_FOREVER);
if (!granted) throw new Error("Calendar access denied");

// Find calendar
const cals = store.calendarsForEntityType($.EKEntityTypeEvent);
let target = null;
for (let i = 0; i < cals.count; i++) {
  const c = cals.objectAtIndex(i);
  if (c.title.js === "Work") { target = c; break; }
}
if (!target) throw new Error("Calendar 'Work' not found");

// Create event
const evt = $.EKEvent.eventWithEventStore(store);
evt.title = "JXA EventKit";
evt.startDate = $.NSDate.dateWithTimeIntervalSinceNow(3600);
evt.endDate = $.NSDate.dateWithTimeIntervalSinceNow(7200);
evt.calendar = target;

// Recurrence: weekly on Mon/Fri
const mon = $.EKRecurrenceDayOfWeek.dayOfWeek(2);
const fri = $.EKRecurrenceDayOfWeek.dayOfWeek(6);
const rule = $.EKRecurrenceRule.alloc.initRecurrenceWithFrequencyIntervalDaysOfTheWeekDaysOfTheMonthMonthsOfTheYearWeeksOfTheYearDaysOfTheYearSetPositionsEnd(
  $.EKRecurrenceFrequencyWeekly,
  1,
  [mon, fri],
  $.nil, $.nil, $.nil, $.nil, $.nil,
  $.nil
);
evt.addRecurrenceRule(rule);

const err = $();
const ok = store.saveEventSpanCommitError(evt, $.EKSpanThisEvent, true, err);
if (!ok) console.log("Save failed: " + err.localizedDescription.js);
```

