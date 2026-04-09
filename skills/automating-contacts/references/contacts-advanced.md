# Contacts JXA Advanced Patterns

## Performance & hybrid filtering
- `.whose` is best for coarse filters; complex OR/NOT can throw -1700.
- Pattern: coarse `.whose` → resolve → JS `.filter` for regex/complex logic.
- Batch-read when possible: `.name()`, `.emails.value()` on a filtered specifier.

## TCC / permissions
- The executing host (Terminal/Script Editor/app bundle) must be granted Contacts access.
- For headless/CI, pre-approve via MDM profile; cannot auto-click the prompt.

## Multi-value write discipline
- Treat emails/phones/addresses/dates as elements; create objects and `.push`.
- Use `Contacts.save()` after meaningful mutations (or after a batch).

## Images via clipboard (ObjC bridge)
Direct `person.image = Path("...")` often fails. Use NSPasteboard to bridge the file:
```javascript
ObjC.import('AppKit');
const pb = $.NSPasteboard.generalPasteboard;
pb.clearContents;
pb.writeObjects($.NSArray.arrayWithObject("/path/to/photo.jpg"));

// Now try setting or use UI scripting to paste
const person = Contacts.people.byName("Ada Lovelace");
// Depending on OS version you may still need a UI paste; clipboard is the safest staging.
```

## Yearless birthdays & date pitfalls
- JXA coerces dates; yearless birthdays become full dates (current year/1604). No clean way to set "yearless" via JXA—accept year or manipulate vCard externally.
- Set custom dates at noon (`T12:00:00`) to avoid timezone rollovers.

## Group membership errors
- Duplicate adds can throw; guard with an existence check (`group.people.whose({ id: {_equals: person.id()} })`).
- Prefer `Contacts.add(person, { to: group })` for clarity; `.people.push` is sugar.

## When the dictionary is insufficient
- ObjC `Contacts.framework` is Swift-heavy and awkward to bridge; fallback is usually:
  - Deprecated `AddressBook` bridge (if still present), or
  - A small external Swift/Python helper invoked via `doShellScript` for rare edge cases.
