# AppleScript `whose` and batching patterns

## Prefer server-side filters
```applescript
tell application "Finder"
  set oldPNGs to (every file of folder "Documents" of home whose name extension is "png" and modification date < ((current date) - 30 * days))
end tell
```

## Batch property reads
```applescript
tell application "Finder"
  set names to name of every file of desktop
end tell
```

## Avoid per-item property reads in loops
- Each property access is an Apple Event (slow).
- Push filters and bulk reads into the app when possible.

