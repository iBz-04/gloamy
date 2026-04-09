# AppleScript performance notes

## Use `whose` to push filtering into the app
```applescript
tell application "Finder"
  set oldImages to (every file of folder "Documents" of home whose name extension is "png" and modification date < ((current date) - 30 * days))
end tell
```

## Avoid large client-side loops when possible
- One large `whose` query is far faster than thousands of per-item property reads.

## Faster list access with a script object
```applescript
script ListContainer
  property fastList : {}
end script

set ListContainer's fastList to bigList
repeat with i from 1 to count of ListContainer's fastList
  set itemValue to item i of ListContainer's fastList
end repeat
```

