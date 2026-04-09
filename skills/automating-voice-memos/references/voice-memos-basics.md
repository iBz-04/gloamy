# Voice Memos JXA Basics (no dictionary)

- Voice Memos is a Mac Catalyst app; **no AppleScript dictionary**. All automation is data-layer + UI scripting.
- Permissions:
  - Full Disk Access to read/write the Group Container and CloudRecordings.db.
  - Accessibility to drive UI/keystrokes.
- Storage paths:
  - Sonoma/Sequoia: `~/Library/Group Containers/group.com.apple.VoiceMemos.shared/Recordings`
  - Older (fallback): `~/Library/Application Support/com.apple.voicememos/Recordings` or `~/Library/Containers/com.apple.VoiceMemos/Data/Library/Application Support/Recordings`
- Database: `CloudRecordings.db` (Core Data SQLite) inside the Recordings folder.
- Core Data epoch offset: `978307200` seconds (Apple epoch). JS date = `(zdate + 978307200)*1000`.
- Prefer data access first; use UI only for actions that must touch the app (record, transcript view).
