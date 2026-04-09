# App dictionary exploration strategies

## Use Script Editor dictionaries
- Script Editor -> File -> Open Dictionary...
- Start with classes (nouns) then commands (verbs).

## Exploration workflow
1) Find the top-level collections (documents, windows, selection).
2) Identify properties (name, id, url, bounds).
3) Test one query at a time in Script Editor.
4) Capture stable identifiers (id) for longer workflows.

## Rapid query patterns
```applescript
tell application "Safari"
  get name of front window
end tell
```

```applescript
tell application "Mail"
  get count of messages of mailbox "INBOX"
end tell
```

## Mapping to JXA
- Use `automating_mac_apps/references/translation-checklist.md`.

