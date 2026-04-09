# AppleScript Basics for Developers (condensed)

## Scope
AppleScript is the discovery and prototyping DSL for Apple Events. Use it to explore app dictionaries, validate commands, and UI script when necessary.

## When to use AppleScript
- Dictionary discovery in Script Editor.
- Rapid prototyping of app commands.
- UI scripting with System Events.
- Small one-off automation.

## When to prefer JXA instead
- Data-heavy logic, JSON pipelines, maintainable scripts.
- Integration with Python/Node.

## Tooling essentials
- Script Editor: dictionary, event log, result pane.
- `osascript`: run inline or file-based scripts.

## Core patterns
```applescript
tell application "Safari"
  set theURL to URL of current tab of front window
end tell
return theURL
```

```applescript
tell application "Finder"
  set names to name of every file of desktop
end tell
return names
```

## Error handling
```applescript
try
  tell application "Finder" to get name of startup disk
on error errMsg number errNum
  log ("Error " & errNum & ": " & errMsg)
end try
```

## UI scripting reliability
- Always `activate` the target app.
- Use `exists` checks + short `delay` loops.
- Prefer named UI elements over index paths.

## AppleScript -> JXA workflow
1) Prototype in AppleScript.
2) Map dictionary terms to JXA access patterns.
3) Port to JXA and return JSON.

