# Voice Memos Data & SQLite Access

## Paths
- Primary (Sonoma/Sequoia): `~/Library/Group Containers/group.com.apple.VoiceMemos.shared/Recordings`
- Legacy fallback: `~/Library/Application Support/com.apple.voicememos/Recordings`
- DB: `CloudRecordings.db` inside the Recordings folder.

Resolve home with ObjC:
```javascript
ObjC.import('Foundation');
const home = $.NSFileManager.defaultManager.homeDirectoryForCurrentUser.path.js;
```

## Core Data timestamps
- Apple epoch offset: `978307200` seconds.
- Convert: `new Date((zdate + 978307200) * 1000)`

## Key tables/columns (CloudRecordings.db)
- `ZCLOUDRECORDING`
  - `ZPATH`: filename (e.g., `20240101-120000.m4a`)
  - `ZCUSTOMLABEL`: user-visible title
  - `ZDATE`: creation timestamp (Core Data epoch)
  - `ZDURATION`: seconds
  - `ZTRASHEDDATE`: non-null => recently deleted
  - `ZFOLDER`: FK to `ZFOLDER.Z_PK`
- `ZFOLDER`
  - `Z_PK`: primary key
  - `ZTITLE`: folder name
  - `ZUUID`: stable UUID

## SQLite query (export-friendly)
```javascript
const sql = `SELECT ZPATH, ZCUSTOMLABEL, ZDATE FROM ZCLOUDRECORDING
 WHERE ZTRASHEDDATE IS NULL;`;
const cmd = `sqlite3 -separator "|" "${dbPath}" "${sql}"`;
const out = app.doShellScript(cmd);
```

## File copy with NSFileManager
```javascript
ObjC.import('Foundation');
const fm = $.NSFileManager.defaultManager;
if (fm.fileExistsAtPath(src)) {
  const ok = fm.copyItemAtPathToPathError(src, dest, null);
}
```

## Safe renaming
- Sanitize title: `title.replace(/[\/\\:]/g, "-")`
- Avoid collisions by appending counter if destination exists.
