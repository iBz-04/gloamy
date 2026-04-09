# Reminders JXA Recipes

## List discovery
```javascript
const app = Application("Reminders");
const lists = app.lists.name(); // ["Inbox", "Personal", ...]
```

## Query active tasks in list
```javascript
const inbox = app.lists.byName("Inbox");
const active = inbox.reminders.whose({ completed: false });
const names = active.name();
```

## Overdue → mark high priority in one batch
```javascript
const now = new Date();
app.reminders.whose({ dueDate: { _lessThan: now }, completed: false }).priority = 1;
```

## Create reminder (constructor + push)
```javascript
const work = app.lists.byName("Work");
const r = app.Reminder({
  name: "Finalize Quarterly Review",
  body: "Include automation metrics",
  priority: 1,
  remindMeDate: new Date()
});
work.reminders.push(r);
```

## Move reminder (copy-delete pattern)
```javascript
function moveReminder(reminder, targetList) {
  const props = {
    name: reminder.name(),
    body: reminder.body(),
    priority: reminder.priority(),
    dueDate: reminder.dueDate(),
    remindMeDate: reminder.remindMeDate(),
    completed: reminder.completed()
  };
  const clone = app.Reminder(props);
  targetList.reminders.push(clone);
  reminder.delete();
}
const src = app.reminders.byName("Finalize Quarterly Review");
moveReminder(src, app.lists.byName("Archive"));
```

## Delete completed items (batch)
```javascript
const done = app.reminders.whose({ completed: true });
app.delete(done);
```

## Snapshot then mutate
```javascript
const dueToday = app.reminders.whose({
  dueDate: { _lessThan: new Date(new Date().setHours(23,59,59,999)) },
  completed: false
});
const ids = dueToday.id();
ids.forEach(id => {
  const r = app.reminders.byId(id);
  r.completed = true;
});
```

## Create follow-ups from transcript text (pattern)
- Upstream meeting workflow can pass parsed action items from Voice Memos transcript into reminders:
```javascript
const items = [
  "Send recap to team",
  "Draft proposal v2",
  "Schedule follow-up demo"
];
const list = app.lists.byName("Follow-ups");
items.forEach(name => {
  const r = app.Reminder({ name });
  list.reminders.push(r);
});
```
- Keep transcript parsing upstream; Reminders should receive clean action strings.
