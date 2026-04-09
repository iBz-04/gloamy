# AppleScript timeouts and async patterns

## Extend Apple Event timeout
```applescript
with timeout of 1800 seconds
  tell application "Xcode" to build workspace document 1
end timeout
```

## Fire-and-forget
```applescript
ignoring application responses
  tell application "Finder" to open application file "Adobe Photoshop.app"
end ignoring
```

