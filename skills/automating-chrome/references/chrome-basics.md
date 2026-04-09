# Chrome JXA basics

## Bootstrapping
```javascript
const chrome = Application("Google Chrome");
chrome.activate();
```

## Required setting (Chrome)
- View -> Developer -> Allow JavaScript from Apple Events

## Chromium variants
- Change app name to match bundle (e.g., "Brave Browser", "Microsoft Edge").
- Verify the scripting dictionary in Script Editor.

