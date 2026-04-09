# Reminders Advanced

## Priority map (integers)
- `0` none, `1` high, `5` medium, `9` low. Use integers; strings like "High" fail.

## Recurrence/location gaps
- Complex recurrence and geofence triggers are not reliably scriptable via JXA.
- Workarounds:
  - Call a Shortcuts workflow that creates recurring/location reminders.
  - Keep a template reminder with the desired recurrence and duplicate via copy-delete.

## Copy-delete move pattern
- No `move` command; container is read-only.
- Copy props → push clone to target list → delete original.
- Caveats: creationDate resets; UUID changes; subtasks must be handled separately if needed.

## Date handling
- Use JS `Date` objects; bridge converts to local `NSDate`.
- "Date-only" due dates still carry a time; set to midnight or noon by convention.

## whose stability
- When changing properties that affect the filter (e.g., completed), resolve IDs first:
  - `const ids = spec.id();` then loop `byId`.
- Batch writes are efficient: setting a property on a specifier sends one Apple Event.

## Debugging
- Use `.properties()` to inspect real property names/casing.
- Common errors:
  - -1700 (type mismatch) → missing `()` or wrong type.
  - -1728 (can't get) → specifier failed; check existence.
  - -10024 (can't make) → use constructor + `.push` instead of `make`.
