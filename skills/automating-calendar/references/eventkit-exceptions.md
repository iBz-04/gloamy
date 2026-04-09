# EventKit exceptions and occurrences (notes)

## Modify a single occurrence
```javascript
// Given an eventIdentifier for a recurring event
const id = "<event-id>";
const master = store.eventWithIdentifier(id);

// Fetch occurrence on a specific date
const cal = $.NSCalendar.currentCalendar;
const comps = cal.componentsFromDate($.NSCalendarUnitYear | $.NSCalendarUnitMonth | $.NSCalendarUnitDay, master.startDate);
// adjust comps to target occurrence date
// build date for that occurrence window, then:
const occ = store.eventWithIdentifier(master.eventIdentifier).copy();
// In practice, use eventStore.eventsMatchingPredicate with date window to get the occurrence
// Then modify and save with EKSpanFutureEvents or EKSpanThisEvent
```

## Cancel a single occurrence
- Find the occurrence (via predicate around that date) and use `store.removeEventSpanCommitError(occ, $.EKSpanThisEvent, true, err)`.

Notes:
- EventKit requires precise date windows to fetch individual occurrences.
- For complex exception sets, enumerate occurrences via predicate and modify/remove selectively.

