# Pages UI scripting patterns

## When to use
- Features missing from the dictionary (inspector toggles, advanced layout).

## Basic pattern
```javascript
const se = Application("System Events");
const pagesProc = se.processes.byName("Pages");
Application("Pages").activate();
delay(0.2);
// Example: open Format sidebar (path varies by version)
// pagesProc.windows[0].toolbars[0].buttons.byName("Format").click();
```

## Notes
- Use Accessibility Inspector to find stable paths.
- Prefer named elements over index paths.
- Add waits around UI element discovery.

