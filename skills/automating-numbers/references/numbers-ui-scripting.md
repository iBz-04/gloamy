# Numbers UI scripting patterns

## When to use
- Only when dictionary access is missing or broken.

## Basic pattern
```javascript
const se = Application("System Events");
const n = se.processes.byName("Numbers");
Application("Numbers").activate();
delay(0.2);
// Example: open Format sidebar (path varies by version)
// n.windows[0].toolbars[0].buttons.byName("Format").click();
```

## Notes
- Use Accessibility Inspector to find stable element paths.
- Prefer named UI elements; avoid index paths.
- Add wait loops before clicking.

