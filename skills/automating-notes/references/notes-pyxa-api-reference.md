# PyXA Notes Module API Reference

> **New in PyXA version 0.0.1** - Control macOS Notes.app using JXA-like syntax from Python.

This reference documents all classes, methods, properties, and enums in the PyXA Notes module. For practical examples and usage patterns, see [notes-recipes.md](notes-recipes.md).

## Contents

- [Class Hierarchy](#class-hierarchy)
- [XANotesApplication](#xanotesapplication)
- [XANote](#xanote)
- [XANotesFolder](#xanotesfolder)
- [XANotesAttachment](#xanotesattachment)
- [XANotesAccount](#xanotesaccount)
- [XANotesDocument](#xanotesdocument)
- [XANotesWindow](#xanoteswindow)
- [List Classes](#list-classes)
- [Enumerations](#enumerations)
- [Quick Reference Tables](#quick-reference-tables)

---

## Class Hierarchy

```
XAObject
├── XANotesApplication (XASBApplication, XACanOpenPath, XACanPrintPath)
│   ├── XANote (XAClipboardCodable, XAShowable, XADeletable)
│   │   └── XANotesAttachment (XAClipboardCodable)
│   ├── XANotesFolder (XAClipboardCodable)
│   ├── XANotesAccount (XAClipboardCodable)
│   ├── XANotesDocument (XAClipboardCodable)
│   └── XANotesWindow (XASBWindow)
└── List Classes
    ├── XANoteList (XAList, XAClipboardCodable)
    ├── XANotesFolderList (XAList, XAClipboardCodable)
    ├── XANotesAttachmentList (XAList, XAClipboardCodable)
    ├── XANotesAccountList (XAList, XAClipboardCodable)
    └── XANotesDocumentList (XAList, XAClipboardCodable)
```

---

## XANotesApplication

**Bases:** `XASBApplication`, `XACanOpenPath`, `XACanPrintPath`

Main entry point for interacting with Notes.app.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `default_account` | `XANotesAccount` | The default account for new notes |
| `frontmost` | `bool` | Whether Notes is the frontmost application |
| `name` | `str` | Application name |
| `selection` | `XANoteList` | Currently selected notes |
| `version` | `str` | Application version |

### Methods

#### `accounts(filter=None) -> XANotesAccountList`

Returns a list of accounts matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
import PyXA
notes = PyXA.Application("Notes")
accounts = notes.accounts()
for acc in accounts:
    print(acc.name)
```

#### `attachments(filter=None) -> XANotesAttachmentList`

Returns all attachments across all notes matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

#### `documents(filter=None) -> XANotesDocumentList`

Returns a list of documents matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

#### `folders(filter=None) -> XANotesFolderList`

Returns all folders matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
folders = notes.folders()
print(folders.name())
# ['Notes', 'Recently Deleted', 'Projects', ...]
```

#### `notes(filter=None) -> XANoteList`

Returns all notes matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
all_notes = notes.notes()
# Filter by name
project_notes = notes.notes({"name": "Project"})
```

#### `new_note(name='New Note', body='', folder=None) -> XANote`

Creates a new note.

**Parameters:**
- `name` (`str`) - Title of the note (default: 'New Note')
- `body` (`str`) - HTML body content (default: '')
- `folder` (`XANotesFolder | None`) - Target folder (default: default folder)

**Returns:** The newly created note

**Example:**
```python
note = notes.new_note(
    name="Meeting Notes",
    body="<h1>Meeting</h1><p>Agenda items...</p>"
)
```

#### `new_folder(name='New Folder', account=None) -> XANotesFolder`

Creates a new folder.

**Parameters:**
- `name` (`str`) - Name of the folder (default: 'New Folder')
- `account` (`XANotesAccount | None`) - Target account (default: default account)

**Returns:** The newly created folder

**Example:**
```python
folder = notes.new_folder(name="Projects")
```

#### `make(specifier, properties=None, data=None) -> XAObject`

Creates a new element without adding to any list. Use `XAList.push()` to add.

**Parameters:**
- `specifier` (`str | ObjectType`) - Class name to create ('note', 'folder')
- `properties` (`dict`) - Properties for the object
- `data` (`Any`) - Initialization data

**Example:**
```python
note = notes.make("note", {"name": "New Note", "body": "<p>Content</p>"})
folder.notes().push(note)
```

#### `open(file_ref) -> XANote`

Opens a file as a note.

**Parameters:**
- `file_ref` (`str | XAPath`) - Path to file to open

**Returns:** The opened note

---

## XANote

**Bases:** `XAObject`, `XAClipboardCodable`, `XAShowable`, `XADeletable`

Represents an individual note in Notes.app.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `body` | `str` | HTML content of the note |
| `container` | `XANotesFolder` | Folder containing this note |
| `creation_date` | `datetime` | When the note was created |
| `id` | `str` | Unique identifier |
| `modification_date` | `datetime` | When the note was last modified |
| `name` | `str` | Title of the note |
| `password_protected` | `bool` | Whether the note is locked |
| `plaintext` | `str` | Plain text content (no HTML) |
| `shared` | `bool` | Whether the note is shared |

### Methods

#### `attachments(filter=None) -> XANotesAttachmentList`

Returns attachments in this note matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
note = notes.notes()[0]
attachments = note.attachments()
for att in attachments:
    print(att.name)
```

#### `move_to(folder) -> XANote`

Moves the note to a different folder.

**Parameters:**
- `folder` (`XANotesFolder`) - Target folder

**Returns:** Self, for method chaining

**Example:**
```python
archive = notes.folders().by_name("Archive")
note.move_to(archive)
```

#### `show() -> XANote`

Shows the note in the Notes main window.

**Returns:** Self, for method chaining

#### `show_separately() -> XANote`

Opens the note in its own separate window.

**Returns:** Self, for method chaining

**Example:**
```python
note.show_separately()  # Opens in new window
```

#### `delete()`

Deletes the note (moves to Recently Deleted).

**Example:**
```python
note.delete()
```

#### `get_clipboard_representation() -> str`

Returns a string representation suitable for the clipboard.

**Returns:** Plain text content of the note

---

## XANotesFolder

**Bases:** `XAObject`, `XAClipboardCodable`

Represents a folder in Notes.app.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `container` | `XANotesAccount` | Account containing this folder |
| `id` | `str` | Unique identifier |
| `name` | `str` | Name of the folder |
| `shared` | `bool` | Whether the folder is shared |

### Methods

#### `notes(filter=None) -> XANoteList`

Returns notes in this folder matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
folder = notes.folders().by_name("Projects")
project_notes = folder.notes()
```

#### `folders(filter=None) -> XANotesFolderList`

Returns subfolders matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
subfolders = folder.folders()
```

#### `move_to(destination) -> XANotesFolder`

Moves the folder to a new location.

**Parameters:**
- `destination` (`XANotesFolder | XANotesAccount`) - Target location

**Returns:** Self, for method chaining

#### `show() -> XANotesFolder`

Shows the folder in the Notes main window.

**Returns:** Self, for method chaining

#### `delete()`

Deletes the folder and its contents.

#### `get_clipboard_representation() -> str`

Returns a string representation suitable for the clipboard.

**Returns:** Folder name

---

## XANotesAttachment

**Bases:** `XAObject`, `XAClipboardCodable`

Represents an attachment in a note.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `container` | `XANote` | Note containing this attachment |
| `content_identifier` | `str` | Content identifier for the attachment |
| `creation_date` | `datetime` | When the attachment was added |
| `id` | `str` | Unique identifier |
| `modification_date` | `datetime` | When the attachment was last modified |
| `name` | `str` | Filename of the attachment |
| `shared` | `bool` | Whether the attachment is shared |
| `url` | `XAURL | None` | URL to the attachment file |

### Methods

#### `save(directory) -> XANotesAttachment`

Saves the attachment to a directory.

**Parameters:**
- `directory` (`str | XAPath`) - Target directory path

**Returns:** Self, for method chaining

**Example:**
```python
attachment = note.attachments()[0]
attachment.save("/Users/me/Downloads")
```

#### `show() -> XANotesAttachment`

Shows the attachment in the Notes main window.

**Returns:** Self, for method chaining

#### `show_separately() -> XANotesAttachment`

Opens the attachment in its own window.

**Returns:** Self, for method chaining

#### `delete()`

Deletes the attachment from the note.

#### `get_clipboard_representation() -> list[NSURL | str]`

Returns a clipboard representation of the attachment.

**Returns:** List containing URL and/or name

---

## XANotesAccount

**Bases:** `XAObject`, `XAClipboardCodable`

Represents an account (iCloud, On My Mac, etc.) in Notes.app.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `default_folder` | `XANotesFolder` | Default folder for new notes |
| `id` | `str` | Unique identifier |
| `name` | `str` | Account name (e.g., "iCloud") |
| `upgraded` | `bool` | Whether the account has been upgraded |

### Methods

#### `folders(filter=None) -> XANotesFolderList`

Returns folders in this account matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
icloud = notes.accounts().by_name("iCloud")
icloud_folders = icloud.folders()
```

#### `notes(filter=None) -> XANoteList`

Returns all notes in this account matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

#### `show() -> XANotesAccount`

Shows the account in the Notes sidebar.

**Returns:** Self, for method chaining

#### `get_clipboard_representation() -> str`

Returns a string representation suitable for the clipboard.

**Returns:** Account name

---

## XANotesDocument

**Bases:** `XAObject`, `XAClipboardCodable`

Represents a Notes document.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `file` | `str` | File location on disk |
| `modified` | `bool` | Whether document has unsaved changes |
| `name` | `str` | Document name |

### Methods

#### `get_clipboard_representation() -> str`

Returns a string representation suitable for the clipboard.

**Returns:** Document name

---

## XANotesWindow

**Bases:** `XASBWindow`

Represents a Notes application window.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `document` | `XANotesDocument` | Document displayed in the window |

---

## List Classes

PyXA provides list wrapper classes with fast enumeration and bulk property access.

### XANoteList

Wrapper for lists of notes with bulk operations.

**Attribute Methods (return lists):**

```python
notes_list = notes.notes()
notes_list.body()              # -> list[str]
notes_list.container()         # -> XANotesFolderList
notes_list.creation_date()     # -> list[datetime]
notes_list.id()                # -> list[str]
notes_list.modification_date() # -> list[datetime]
notes_list.name()              # -> list[str]
notes_list.password_protected()# -> list[bool]
notes_list.plaintext()         # -> list[str]
notes_list.shared()            # -> list[bool]
notes_list.attachments()       # -> XANotesAttachmentList
```

**Filter Methods:**

```python
notes_list.by_body(body)
notes_list.by_container(container)
notes_list.by_creation_date(creation_date)
notes_list.by_id(id)
notes_list.by_modification_date(modification_date)
notes_list.by_name(name)
notes_list.by_password_protected(password_protected)
notes_list.by_plaintext(plaintext)
notes_list.by_shared(shared)
```

**Action Methods:**

```python
notes_list.show_separately()              # -> XANoteList
notes_list.get_clipboard_representation() # -> list[str]
```

### XANotesFolderList

Wrapper for lists of folders with bulk operations.

**Attribute Methods:**

```python
folders = notes.folders()
folders.container()  # -> XANotesAccountList
folders.folders()    # -> XANotesFolderList (subfolders)
folders.id()         # -> list[str]
folders.name()       # -> list[str]
folders.notes()      # -> XANoteList
folders.shared()     # -> list[bool]
```

**Filter Methods:**

```python
folders.by_container(container)
folders.by_id(id)
folders.by_name(name)
folders.by_shared(shared)
```

**Action Methods:**

```python
folders.get_clipboard_representation()  # -> list[str]
```

### XANotesAttachmentList

Wrapper for lists of attachments with bulk operations.

**Attribute Methods:**

```python
attachments = note.attachments()
attachments.container()          # -> XANoteList
attachments.content_identifier() # -> list[str]
attachments.creation_date()      # -> list[datetime]
attachments.id()                 # -> list[str]
attachments.modification_date()  # -> list[datetime]
attachments.name()               # -> list[str]
attachments.shared()             # -> list[bool]
attachments.url()                # -> list[XAURL | None]
```

**Filter Methods:**

```python
attachments.by_container(container)
attachments.by_content_identifier(content_identifier)
attachments.by_creation_date(creation_date)
attachments.by_id(id)
attachments.by_modification_date(modification_date)
attachments.by_name(name)
attachments.by_shared(shared)
attachments.by_url(url)
```

**Action Methods:**

```python
attachments.save(directory)  # -> XANotesAttachmentList
```

### XANotesAccountList

Wrapper for lists of accounts with bulk operations.

**Attribute Methods:**

```python
accounts = notes.accounts()
accounts.default_folder()  # -> XANotesFolderList
accounts.folders()         # -> XANotesFolderList
accounts.id()              # -> list[str]
accounts.name()            # -> list[str]
accounts.notes()           # -> XANoteList
accounts.upgraded()        # -> list[bool]
```

**Filter Methods:**

```python
accounts.by_default_folder(default_folder)
accounts.by_id(id)
accounts.by_name(name)
accounts.by_upgraded(upgraded)
```

**Action Methods:**

```python
accounts.get_clipboard_representation()  # -> list[str]
```

### XANotesDocumentList

Wrapper for lists of documents with bulk operations.

**Attribute Methods:**

```python
docs = notes.documents()
docs.file()      # -> list[str]
docs.modified()  # -> list[bool]
docs.name()      # -> list[str]
```

**Filter Methods:**

```python
docs.by_file(file)
docs.by_modified(modified)
docs.by_name(name)
```

**Action Methods:**

```python
docs.get_clipboard_representation()  # -> list[str]
```

---

## Enumerations

### FileFormat

File format options.

| Value | Raw Value | Description |
|-------|-----------|-------------|
| `NATIVE` | 1769235821 | Native Notes format |

### ObjectType

Creatable object types.

| Value | Description |
|-------|-------------|
| `ACCOUNT` | Notes account |
| `ATTACHMENT` | Note attachment |
| `FOLDER` | Notes folder |
| `NOTE` | Individual note |

---

## Quick Reference Tables

### Common Operations

| Task | Code |
|------|------|
| Get Notes app | `notes = PyXA.Application("Notes")` |
| Get all notes | `all_notes = notes.notes()` |
| Get note by name | `note = notes.notes().by_name("Title")` |
| Create note | `note = notes.new_note(name="Title", body="<p>Content</p>")` |
| Create folder | `folder = notes.new_folder(name="Projects")` |
| Get iCloud account | `icloud = notes.accounts().by_name("iCloud")` |
| Get folder notes | `folder_notes = folder.notes()` |
| Move note | `note.move_to(target_folder)` |
| Delete note | `note.delete()` |
| Show note | `note.show()` |
| Open in new window | `note.show_separately()` |
| Save attachment | `attachment.save("/path/to/directory")` |
| Get plain text | `text = note.plaintext` |
| Get HTML body | `html = note.body` |

### Property Access Patterns

```python
# Single object
note = notes.notes()[0]
print(note.name)
print(note.body)
print(note.creation_date)

# Bulk access on lists
all_notes = notes.notes()
print(all_notes.name())           # Returns list[str]
print(all_notes.creation_date())  # Returns list[datetime]
print(all_notes.shared())         # Returns list[bool]

# Filtering
shared_notes = all_notes.by_shared(True)
recent = all_notes.by_modification_date(datetime.now())
```

### Account and Folder Navigation

```python
# Navigate hierarchy: Account -> Folders -> Notes
icloud = notes.accounts().by_name("iCloud")
projects = icloud.folders().by_name("Projects")
project_notes = projects.notes()

# Get subfolders
subfolders = projects.folders()

# Get all notes in account
all_icloud_notes = icloud.notes()
```

### Working with Attachments

```python
# Get attachments from a note
note = notes.notes()[0]
attachments = note.attachments()

# Save all attachments
for att in attachments:
    att.save("/Downloads")

# Bulk save
attachments.save("/Downloads")

# Filter by name
images = attachments.by_name(".png")
```

---

## See Also

- [PyXA Notes Documentation](https://skaplanofficial.github.io/PyXA/reference/apps/notes.html) - Official PyXA documentation
- [notes-recipes.md](notes-recipes.md) - Practical usage examples
- [notes-basics.md](notes-basics.md) - JXA fundamentals
- [notes-advanced.md](notes-advanced.md) - Advanced patterns and HTML handling
- [notes-dictionary.md](notes-dictionary.md) - Complete property reference
