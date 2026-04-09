# Notes Dictionary & Types

## Core classes
- `Application("Notes")`
- Elements: `accounts`, `folders`, `notes`
- `Account` → properties: `name`, `id`; elements: `folders`
- `Folder` → properties: `name`, `id`, `container`; elements: `folders`, `notes`
- `Note` → properties: `name`, `body` (HTML), `creationDate`, `modificationDate`, `id`, `container`

## Commands
- Create: `Notes.Note({...})` + `folder.notes.push(obj)` or `Notes.make({ new: 'note', at: folder, withProperties: {...} })`
- Move: `Notes.move(noteSpecifier, { to: folderSpecifier })`
- Delete: `note.delete()` or `folder.notes.whose(...).delete()`

## Filters
- `.whose` with operators: `_contains`, `_beginsWith`, `_endsWith`, `_greaterThan`, `_lessThan`, `_equals`, `_not`, `_and`, `_or`
- Examples:
  - `app.notes.whose({ name: { _contains: "Draft" } })`
  - `folder.notes.whose({ creationDate: { _greaterThan: someDate } })`

## Reading/writing
- Read with methods: `note.name()`, `note.body()`, `note.id()`
- Write with assignment: `note.body = "<h1>Title</h1>...";`
