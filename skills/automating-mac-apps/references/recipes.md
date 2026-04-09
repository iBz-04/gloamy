# Recipes and side-by-side patterns

## Contents
- AppleScript vs JXA: same Apple Event, different syntax
- Finder selection names
- Hybrid workflow (recommended)
- Pipeline pattern (Python + JXA)
- Helper library (JXA)
- Helper library (AppleScript)
- Worked example: AppleScript discovery -> JXA production
- Validation pattern

## AppleScript vs JXA: same Apple Event, different syntax
### Safari current tab URL
```applescript
tell application "Safari"
  set theURL to URL of current tab of front window
end tell
return theURL
```

```javascript
const safari = Application("Safari");
safari.windows[0].currentTab().url();
```

## Finder selection names
```applescript
tell application "Finder"
  set sel to selection
  if sel is {} then return "No selection"
  return name of sel
end tell
```

```javascript
const finder = Application("Finder");
const sel = finder.selection();
const out = sel.length
  ? { names: sel.map(f => f.name()) }
  : { error: "No selection" };
JSON.stringify(out);
```

## Hybrid workflow (recommended)
1) Explore dictionary in Script Editor (AppleScript terms).
2) Prototype in AppleScript for quick discovery.
3) Port to JXA for production logic and JSON output.

## Pipeline pattern (Python + JXA)
- Python orchestrates.
- JXA handles app automation and returns JSON.

## Helper library (JXA)
Use the reusable helper template in `automating_mac_apps/references/jxa-helpers.js`.

## Helper library (AppleScript)
```applescript
on safeGet(taskScript, fallback)
  try
    return taskScript's run()
  on error
    return fallback
  end try
end safeGet

script ReadStartupDisk
  on run()
    tell application "Finder" to get name of startup disk
  end run
end script

-- Example usage:
-- set result to safeGet(ReadStartupDisk, "Unknown")
```

## Worked example: AppleScript discovery -> JXA production

### Step 1: AppleScript prototype (discover terms)
```applescript
tell application "Finder"
  if selection is {} then return "No selection"
  set selNames to name of selection
end tell
return selNames
```

### Step 2: Port to JXA (production + JSON)
```javascript
const finder = Application("Finder");
const sel = finder.selection();
const out = sel.length
  ? { names: sel.map(f => f.name()) }
  : { error: "No selection" };
JSON.stringify(out);
```

### Step 3: Use from Python (pipeline)
```python
import json, subprocess

out = subprocess.check_output(
  ["osascript", "-l", "JavaScript", "get_finder_selection.js"],
  text=True
)
data = json.loads(out)
```

## Validation pattern
Use a read-only probe before destructive automation.
```javascript
const finder = Application("Finder");
const count = finder.selection().length;
JSON.stringify({ selectionCount: count });
```
