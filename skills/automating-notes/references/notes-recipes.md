# Notes JXA Recipes

## Ensure folder path (account-based)

**JXA:**
```javascript
function ensurePath(acc, path) {
  const parts = path.split("/").filter(Boolean);
  let container = acc;
  parts.forEach(seg => {
    let f;
    try { f = container.folders.byName(seg); f.name(); }
    catch (e) {
      f = Notes.Folder({ name: seg });
      container.folders.push(f);
    }
    container = f;
  });
  return container;
}

const acc = Notes.accounts.byName("iCloud");
const meetingsFolder = ensurePath(acc, "meetings/Acme/2024-07-01-Review");
```

**PyXA:**
```python
import PyXA

def ensure_path(account, path):
    """Ensure folder path exists, creating intermediate folders as needed"""
    parts = [p for p in path.split("/") if p]  # Filter empty parts
    container = account

    for part in parts:
        try:
            # Try to find existing folder
            folder = container.folders().by_name(part)
            folder.name()  # Verify access
        except:
            # Folder doesn't exist, create it
            folder = PyXA.Application("Notes").make("folder", {"name": part})
            container.folders().push(folder)

        container = folder

    return container

# Usage
notes = PyXA.Application("Notes")
icloud_account = notes.accounts().by_name("iCloud")
meetings_folder = ensure_path(icloud_account, "meetings/Acme/2024-07-01-Review")
```

## Create note with HTML body

**JXA:**
```javascript
const note = Notes.Note({
  name: "Client Review",
  body: "<h1>Client Review</h1><div>Agenda...</div><h2>Decisions</h2><ul><li>...</li></ul>"
});
meetingsFolder.notes.push(note);
```

**PyXA:**
```python
import PyXA

# Create note with HTML body (using the meetings folder from above)
html_body = """
<h1>Client Review</h1>
<div>Agenda items to discuss...</div>
<h2>Decisions</h2>
<ul>
<li>Decision 1: ...</li>
<li>Decision 2: ...</li>
</ul>
"""

note = meetings_folder.notes().push({
    "name": "Client Review",
    "body": html_body
})

print(f"Created note: {note.name()}")
```

## Query recent notes

**JXA:**
```javascript
const yesterday = new Date(Date.now() - 86400*1000);
const recent = meetingsFolder.notes.whose({ creationDate: { _greaterThan: yesterday } });
const names = recent.name();
```

**PyXA:**
```python
import PyXA
from datetime import datetime, timedelta

# Query notes created in the last 24 hours
yesterday = datetime.now() - timedelta(days=1)

# Filter notes by creation date
recent_notes = meetings_folder.notes().filter(
    lambda note: note.creation_date() > yesterday
)

# Get names of recent notes
recent_names = [note.name() for note in recent_notes]

print(f"Recent notes: {recent_names}")
```

## Move note

**JXA:**
```javascript
const target = ensurePath(acc, "Archive/2024");
Notes.move(meetingsFolder.notes.byName("Client Review"), { to: target });
```

**PyXA:**
```python
# Move note to archive folder
archive_folder = ensure_path(icloud_account, "Archive/2024")
note_to_move = meetings_folder.notes().by_name("Client Review")

if note_to_move:
    note_to_move.move_to(archive_folder)
    print("Note moved to archive")
else:
    print("Note not found")
```

## Checklist workaround (UI)
- Create note text, then (if needed) front Notes and send `Cmd+Shift+L` via System Events to toggle checklist on selected lines.

## People dossiers
- Save under `people/<first>-<last>/<date>-<title>` and `people/<first>-<last>/overview`.
```javascript
const personFolder = ensurePath(acc, "people/Ada-Lovelace");
const overview = ensurePath(acc, "people/Ada-Lovelace/overview");
const dossier = Notes.Note({
  name: "Overview",
  body: "<h1>Ada Lovelace</h1><div>Birthday: ...</div><div>Allergies: ...</div>"
});
overview.notes.push(dossier);
```
