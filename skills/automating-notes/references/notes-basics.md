# Notes JXA Basics

## Initialize
```javascript
const Notes = Application("Notes");
Notes.includeStandardAdditions = true;
```

## Hierarchy & specifiers
- Application → Accounts → Folders → Notes (folders can nest).
- `Notes.accounts.byName("iCloud")` → account specifier.
- `account.folders.byName("Work")` → folder specifier.
- `folder.notes` → notes specifier.
- Use methods to read: `note.name()`, `note.body()`.

## Create note (constructor + push)
```javascript
const acc = Notes.accounts.byName("iCloud");
const f = acc.folders.byName("Work");
const n = Notes.Note({
  name: "Meeting Notes",
  body: "<h1>Team Sync</h1><div>Decisions...</div>"
});
f.notes.push(n);
```

## Ensure folder path
- Iterate path segments, check `folders.byName`, create if missing (see recipes).

## Move note
```javascript
Notes.move(noteSpecifier, { to: targetFolder });
```

## Query
- Server-side filter: `folder.notes.whose({ name: { _contains: "Q3" } })`.
- Access properties on specifier to batch-read: `.name()`, `.id()`.
