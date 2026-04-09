# AppleScript basics (DSL for Apple Events)

## Contents
- Mental model
- Core patterns
- Dictionary discovery
- Control flow + errors
- Shell interop
- UI scripting (last resort)
- UI scripting reliability helpers
- AppleScript strengths

## Mental model
- `tell application "App"` sends Apple Events to a scriptable app.
- Most results are **references**; ask for properties to get values.
- Dictionaries define the nouns (classes) and verbs (commands).

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

## Dictionary discovery
- Script Editor > File > Open Dictionary...
- Use dictionary terms exactly (class names, properties, commands).

## Control flow + errors
```applescript
try
  tell application "Finder" to get name of startup disk
on error errMsg number errNum
  log ("Error " & errNum & ": " & errMsg)
end try
```

## Shell interop
```applescript
set dirPath to "/tmp"
set cmd to "ls -la " & quoted form of dirPath
set out to do shell script cmd
return out
```

## UI scripting (last resort)
```applescript
tell application "System Events"
  tell process "Safari"
    click menu item "New Tab" of menu 1 of menu bar item "File" of menu bar 1
  end tell
end tell
```

## UI scripting reliability helpers
```applescript
on waitForProcess(procName, timeoutSeconds)
  set endTime to (current date) + timeoutSeconds
  tell application "System Events"
    repeat until (exists process procName) or ((current date) > endTime)
      delay 0.2
    end repeat
    if not (exists process procName) then
      error "Timed out waiting for process " & procName
    end if
  end tell
end waitForProcess
```

```applescript
on waitForUIElement(uiRef, timeoutSeconds)
  set endTime to (current date) + timeoutSeconds
  repeat until (exists uiRef) or ((current date) > endTime)
    delay 0.2
  end repeat
  if not (exists uiRef) then error "Timed out waiting for UI element"
end waitForUIElement
```

Tips:
- Always `activate` the target app and wait for its process.
- Use `exists` checks and short `delay` loops instead of fixed sleeps.
- Prefer menu items and named buttons over index-based UI paths.

## AppleScript strengths
- Fast discovery with Script Editor.
- Compact UI scripting patterns.
- Lots of legacy examples.
