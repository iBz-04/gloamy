# Voice Memos UI Automation (System Events)

- No dictionary: rely on System Events + keyboard shortcuts (prefer shortcuts over clicks).
- Accessibility tree (Catalyst): expect deep `AXGroup`/`AXSplitGroup` nesting; buttons may lack titles, use role/description.

## Keyboard shortcuts (preferred)
- Start Recording: `Cmd+N`
- Stop/Done: `Cmd+.` or Space
- Play/Pause: Space
- Trim: `Cmd+T`
- Delete: `Delete` key (keyCode 51)
- Enhance Audio: `Shift+Cmd+E`
- Skip Silence: `Shift+Cmd+X`

Example keystroke:
```javascript
const se = Application("System Events");
se.keystroke("n", {using: "command down"}); // start recording
```

## Recursive finder (simplified pattern)
Use a stack/DFS to find elements by role/description when needed:
```javascript
function findUIElement(root, criteria) {
  const stack = [root];
  while (stack.length) {
    const el = stack.pop();
    let ok = true;
    for (const key in criteria) {
      try {
        if (el[key]()!== criteria[key]) { ok = false; break; }
      } catch (e) { ok = false; break; }
    }
    if (ok) return el;
    try {
      el.uiElements().forEach(child => stack.push(child));
    } catch (e) {}
  }
  return null;
}
```

## Transcription view (UI)
- Open the app frontmost, search via `Cmd+F`, Enter, then use View > Transcript (if present) or locate a button with description like "Transcript".
- Scrape text by finding a large `AXTextArea` after the transcript pane is shown.

## UI fragility
- Sidebar hidden changes group indices; avoid hardcoded indices.
- Always ensure Accessibility permission is granted before automating.
